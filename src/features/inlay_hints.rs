use crate::ast::{Layout, NodeType, PropertyValue, TypeValueInner};
use crate::document::Document;
use crate::schema;
use tower_lsp::lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Range};

pub fn get_inlay_hints(doc: &Document, range: Range) -> Vec<InlayHint> {
    let Some(layout) = doc.layout.as_ref() else {
        return vec![];
    };

    let start = doc.position_to_offset(range.start);
    let end = doc.position_to_offset(range.end);

    let mut hints = vec![];
    for alias in &layout.aliases {
        hints_in_type_value_inner(
            &alias.target.inner,
            &alias.target.name,
            start,
            end,
            doc,
            &layout,
            &mut hints,
        );
    }
    hints_in_node(&layout.root, start, end, doc, layout, &mut hints);
    hints
}

fn hints_in_node(
    node: &NodeType,
    start: usize,
    end: usize,
    doc: &Document,
    layout: &Layout,
    hints: &mut Vec<InlayHint>,
) {
    for prop in &node.properties {
        if prop.span.start < start || prop.span.end > end {
            continue;
        }
        hints_in_value(&prop.value, &prop.key, &node.name, doc, layout, hints);
    }

    for child in &node.children {
        hints_in_node(child, start, end, doc, layout, hints);
    }
}

fn hints_in_type_value_inner(
    inner: &TypeValueInner,
    type_name: &str,
    start: usize,
    end: usize,
    doc: &Document,
    layout: &Layout,
    hints: &mut Vec<InlayHint>,
) {
    if let TypeValueInner::Properties(props) = inner {
        for prop in props {
            if prop.span.start < start || prop.span.end > end {
                continue;
            }
            hints_in_value(&prop.value, &prop.key, type_name, doc, layout, hints);
        }
    }
}

fn hints_in_value(
    value: &PropertyValue,
    prop_key: &str,
    parent_name: &str,
    doc: &Document,
    layout: &Layout,
    hints: &mut Vec<InlayHint>,
) {
    match value {
        PropertyValue::UInt(_, span) => {
            let real = crate::utils::resolve_real_alias_name(parent_name, &layout.aliases);
            if let Some(unit) = unit_for_prop(real, prop_key) {
                hints.push(InlayHint {
                    position: doc.offset_to_position(span.end),
                    label: InlayHintLabel::String(unit.to_string()),
                    kind: Some(InlayHintKind::TYPE),
                    padding_left: Some(true),
                    padding_right: Some(false),
                    text_edits: None,
                    tooltip: None,
                    data: None,
                });
            }
        }

        PropertyValue::Type(tv) => {
            hints_in_type_value_inner(&tv.inner, &tv.name, 0, usize::MAX, doc, layout, hints);

            if let TypeValueInner::Value(inner_val) = &tv.inner {
                if let PropertyValue::UInt(_, span) = inner_val.as_ref() {
                    if let Some(hint) = constructor_hint(&tv.name) {
                        hints.push(InlayHint {
                            position: doc.offset_to_position(span.end),
                            label: InlayHintLabel::String(hint.to_string()),
                            kind: Some(InlayHintKind::TYPE),
                            padding_left: Some(true),
                            padding_right: Some(false),
                            text_edits: None,
                            tooltip: None,
                            data: None,
                        });
                    }
                }
            }
        }

        PropertyValue::Literal(_, _) => {}
    }
}

fn unit_for_prop(type_name: &str, prop_key: &str) -> Option<&'static str> {
    let def = schema::lookup(type_name)?;
    let _ = def.find_prop(prop_key)?;

    match (type_name, prop_key) {
        (_, "max_width") => Some("px"),
        (_, "min_width") => Some("px"),
        (_, "max_height") => Some("px"),
        (_, "min_height") => Some("px"),
        (_, "max_size") => Some("px"),
        (_, "font_size") => Some("pt"),
        (_, "line_spacing") => Some("px"),
        ("Spacing", "top") => Some("px"),
        ("Spacing", "right") => Some("px"),
        ("Spacing", "bottom") => Some("px"),
        ("Spacing", "left") => Some("px"),
        ("Border", "size") => Some("px"),
        ("Border", "radius") => Some("px"),
        _ => None,
    }
}

fn constructor_hint(type_name: &str) -> Option<&'static str> {
    match type_name {
        "Spacing" => Some("all sides"),
        _ => None,
    }
}
