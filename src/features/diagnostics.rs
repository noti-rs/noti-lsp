use crate::ast::{
    AliasDeclaration, Layout, NodeType, Property, PropertyValue, Span, TypeValue, TypeValueInner,
};
use crate::consts::LSP_NAME;
use crate::document::Document;
use crate::schema::{self, ValueKind};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn make_diagnostics(doc: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];

    // Syntax errors from pest
    for err in &doc.errors {
        let pos = doc.offset_to_position(err.offset);
        diagnostics.push(Diagnostic {
            range: Range {
                start: pos,
                end: Position {
                    line: pos.line,
                    character: pos.character + 1,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some(LSP_NAME.to_string()),
            message: err.message.clone(),
            ..Default::default()
        });
    }

    // Semantic diagnostics
    if let Some(layout) = &doc.layout {
        check_layout(layout, doc, &mut diagnostics);
    }

    diagnostics
}

fn check_layout(layout: &Layout, doc: &Document, diagnostics: &mut Vec<Diagnostic>) {
    for alias in &layout.aliases {
        check_alias(alias, layout, doc, diagnostics);
    }
    check_node(&layout.root, layout, doc, diagnostics);
}

fn check_alias(
    alias: &AliasDeclaration,
    layout: &Layout,
    doc: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let real = crate::utils::resolve_real_alias_name(&alias.target.name, &layout.aliases);
    if schema::lookup(real).is_none() {
        diagnostics.push(error(
            doc,
            &alias.target.name_span,
            format!("Unknown type `{}`", alias.target.name),
        ));
        return;
    }

    check_type_value(&alias.target, layout, doc, diagnostics);
}

fn check_node(node: &NodeType, layout: &Layout, doc: &Document, diagnostics: &mut Vec<Diagnostic>) {
    let real = crate::utils::resolve_real_alias_name(&node.name, &layout.aliases);

    let Some(type_def) = schema::lookup(real) else {
        diagnostics.push(error(
            doc,
            &node.name_span,
            format!("Unknown component `{}`", node.name),
        ));

        // Still check children even if parent is unknown
        for child in &node.children {
            check_node(child, layout, doc, diagnostics);
        }

        return;
    };

    let mut seen_keys: Vec<&str> = vec![];
    for prop in &node.properties {
        // Duplicate property
        if seen_keys.contains(&prop.key.as_str()) {
            diagnostics.push(warning(
                doc,
                &prop.key_span,
                format!("Duplicate property `{}`", prop.key),
            ));
        } else {
            seen_keys.push(&prop.key);
        }

        check_property(prop, real, layout, doc, diagnostics);
    }

    for child in &node.children {
        check_node(child, layout, doc, diagnostics);
    }
}

fn check_property(
    prop: &Property,
    parent_type: &str,
    layout: &Layout,
    doc: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(type_def) = schema::lookup(parent_type) else {
        return;
    };

    let Some(prop_def) = type_def.find_prop(&prop.key) else {
        diagnostics.push(error(
            doc,
            &prop.key_span,
            format!("Unknown property `{}` on `{}`", prop.key, parent_type),
        ));
        return;
    };

    check_value_kind(
        &prop.value,
        &prop_def.value,
        &prop.key,
        parent_type,
        layout,
        doc,
        diagnostics,
    );
}

fn check_value_kind(
    value: &PropertyValue,
    expected: &ValueKind,
    prop_key: &str,
    parent_type: &str,
    layout: &Layout,
    doc: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match (value, expected) {
        // UInt expected
        (PropertyValue::UInt(_, _), ValueKind::UInt) => {}

        // Nested type expected
        (PropertyValue::Type(tv), ValueKind::Type(expected_type)) => {
            let real = crate::utils::resolve_real_alias_name(&tv.name, &layout.aliases);
            let real_expected =
                crate::utils::resolve_real_alias_name(expected_type, &layout.aliases);
            if real != real_expected {
                diagnostics.push(error(
                    doc,
                    &tv.name_span,
                    format!(
                        "Expected `{expected_type}` for `{prop_key}`, found `{}`",
                        tv.name
                    ),
                ));
            } else {
                check_type_value(tv, layout, doc, diagnostics);
            }
        }

        // Enum expected — must be a Literal with a valid variant
        (PropertyValue::Literal(lit, span), ValueKind::Enum(variants)) => {
            if !variants.contains(&lit.as_str()) {
                diagnostics.push(error(
                    doc,
                    span,
                    format!(
                        "Invalid value `{lit}` for `{prop_key}` on `{parent_type}`\n\
                         Expected one of: {}",
                        variants.join(", ")
                    ),
                ));
            }
        }

        (PropertyValue::Literal(_, _), ValueKind::Literal) => {}

        // Constructor value (e.g. Spacing(10)) — check if type accepts it
        (PropertyValue::Type(tv), ValueKind::UInt) => {
            // e.g. someone wrote `spacing = 10` where a TypeValue is expected
            diagnostics.push(error(
                doc,
                &tv.span,
                format!(
                    "Expected an integer for `{prop_key}`, found a type `{}`",
                    tv.name
                ),
            ));
        }

        (PropertyValue::UInt(_, span), ValueKind::Type(expected_type)) => {
            diagnostics.push(error(
                doc,
                span,
                format!("Expected `{expected_type}(...)` for `{prop_key}`, found an integer"),
            ));
        }

        (PropertyValue::UInt(_, span), ValueKind::Enum(variants)) => {
            diagnostics.push(error(
                doc,
                span,
                format!(
                    "Expected one of [{}] for `{prop_key}`, found an integer",
                    variants.join(", ")
                ),
            ));
        }

        (PropertyValue::Literal(lit, span), ValueKind::UInt) => {
            diagnostics.push(error(
                doc,
                span,
                format!("Expected an integer for `{prop_key}`, found `{lit}`"),
            ));
        }

        (PropertyValue::Literal(lit, span), ValueKind::Type(expected_type)) => {
            diagnostics.push(error(
                doc,
                span,
                format!("Expected `{expected_type}(...)` for `{prop_key}`, found `{lit}`"),
            ));
        }

        _ => {}
    }
}

fn check_type_value(
    tv: &TypeValue,
    layout: &Layout,
    doc: &Document,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let real = crate::utils::resolve_real_alias_name(&tv.name, &layout.aliases);
    let Some(type_def) = schema::lookup(real) else {
        diagnostics.push(error(
            doc,
            &tv.name_span,
            format!("Unknown type `{}`", tv.name),
        ));
        return;
    };

    match &tv.inner {
        TypeValueInner::Properties(props) => {
            let mut seen: Vec<&str> = vec![];
            for prop in props {
                if seen.contains(&prop.key.as_str()) {
                    diagnostics.push(warning(
                        doc,
                        &prop.key_span,
                        format!("Duplicate property `{}`", prop.key),
                    ));
                } else {
                    seen.push(&prop.key);
                }
                check_property(prop, real, layout, doc, diagnostics);
            }
        }

        TypeValueInner::Value(val) => {
            // Positional constructor e.g. Spacing(10)
            if let Some(constructor_kind) = &type_def.constructor {
                check_value_kind(
                    val,
                    constructor_kind,
                    &tv.name,
                    real,
                    layout,
                    doc,
                    diagnostics,
                );
            } else {
                diagnostics.push(error(
                    doc,
                    &tv.span,
                    format!("`{}` does not accept a positional value", tv.name),
                ));
            }
        }

        TypeValueInner::Empty => {}
    }
}

fn error(doc: &Document, span: &Span, message: String) -> Diagnostic {
    Diagnostic {
        range: span.to_range(doc),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(LSP_NAME.to_string()),
        message,
        ..Default::default()
    }
}

fn warning(doc: &Document, span: &Span, message: String) -> Diagnostic {
    Diagnostic {
        range: span.to_range(doc),
        severity: Some(DiagnosticSeverity::WARNING),
        source: Some(LSP_NAME.to_string()),
        message,
        ..Default::default()
    }
}
