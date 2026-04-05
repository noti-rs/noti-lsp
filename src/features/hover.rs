use crate::ast::{
    AliasDeclaration, Layout, NodeType, Property, PropertyValue, Span, TypeValue, TypeValueInner,
};
use crate::document::Document;
use crate::schema;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

pub fn get_hover(doc: &Document, pos: Position) -> Option<Hover> {
    let offset = doc.position_to_offset(pos);
    let layout = doc.layout.as_ref()?;
    hover_in_layout(layout, offset)
}

fn hover_in_layout(layout: &Layout, offset: usize) -> Option<Hover> {
    // Check alias declarations first
    for alias in &layout.aliases {
        if let Some(h) = hover_in_alias(alias, offset) {
            return Some(h);
        }
    }
    hover_in_node(&layout.root, offset, &layout.aliases)
}

fn hover_in_alias(alias: &AliasDeclaration, offset: usize) -> Option<Hover> {
    // Hovering the alias name itself
    if span_contains(&alias.name_span, offset) {
        let target_type = schema::lookup(&alias.target.name);
        let base_desc = target_type
            .map(|t| t.description)
            .unwrap_or("User-defined alias");

        return Some(make_hover(format!(
            "**alias** `{name}` = `{target}(...)`\n\n{base_desc}",
            name = alias.name,
            target = alias.target.name,
        )));
    }

    // Hovering something inside the alias target
    hover_in_type_value(&alias.target, offset, &[])
}

fn hover_in_node(node: &NodeType, offset: usize, aliases: &[AliasDeclaration]) -> Option<Hover> {
    // Hovering the component name
    if span_contains(&node.name_span, offset) {
        return Some(hover_for_type_name(&node.name, aliases));
    }

    // Hovering a property key or value
    for prop in &node.properties {
        if let Some(h) = hover_in_property(prop, offset, &node.name, aliases) {
            return Some(h);
        }
    }

    // Recurse into children
    for child in &node.children {
        if let Some(h) = hover_in_node(child, offset, aliases) {
            return Some(h);
        }
    }

    None
}

fn hover_in_property(
    prop: &Property,
    offset: usize,
    parent_name: &str,
    aliases: &[AliasDeclaration],
) -> Option<Hover> {
    // Hovering the key
    if span_contains(&prop.key_span, offset) {
        return Some(hover_for_prop_key(&prop.key, parent_name, aliases));
    }

    // Hovering inside the value
    hover_in_property_value(&prop.value, offset, &prop.key, parent_name, aliases)
}

fn hover_in_property_value(
    value: &PropertyValue,
    offset: usize,
    prop_key: &str,
    parent_name: &str,
    aliases: &[AliasDeclaration],
) -> Option<Hover> {
    match value {
        PropertyValue::Type(tv) => hover_in_type_value(tv, offset, aliases),

        PropertyValue::Literal(lit, span) => {
            if span_contains(span, offset) {
                return Some(hover_for_literal(lit, prop_key, parent_name, aliases));
            }
            None
        }

        PropertyValue::UInt(n, span) => {
            if span_contains(span, offset) {
                return Some(make_hover(format!("`{n}` — unsigned integer")));
            }
            None
        }
    }
}

fn hover_in_type_value(
    tv: &TypeValue,
    offset: usize,
    aliases: &[AliasDeclaration],
) -> Option<Hover> {
    if span_contains(&tv.name_span, offset) {
        return Some(hover_for_type_name(&tv.name, aliases));
    }

    if let TypeValueInner::Properties(props) = &tv.inner {
        for prop in props {
            if let Some(h) = hover_in_property(prop, offset, &tv.name, aliases) {
                return Some(h);
            }
        }
    }

    None
}

fn hover_for_type_name(name: &str, aliases: &[AliasDeclaration]) -> Hover {
    // Check schema first
    if let Some(def) = schema::lookup(name) {
        let props: Vec<String> = def
            .props
            .iter()
            .map(|p| format!("- `{}` — {}", p.name, p.description))
            .collect();

        let body = if props.is_empty() {
            def.description.to_string()
        } else {
            format!(
                "{}\n\n**Properties:**\n{}",
                def.description,
                props.join("\n")
            )
        };

        return make_hover(format!("**`{name}`**\n\n{body}"));
    }

    // Check aliases
    if let Some(alias) = aliases.iter().find(|a| a.name == name) {
        return make_hover(format!("**alias** `{name}` = `{}(...)`", alias.target.name));
    }

    make_hover(format!("`{name}` — unknown type"))
}

fn hover_for_prop_key(key: &str, parent_name: &str, aliases: &[AliasDeclaration]) -> Hover {
    // Resolve the real type (follow alias if needed)
    let real_name = resolve_real_name(parent_name, aliases);

    if let Some(def) = schema::lookup(real_name) {
        if let Some(prop) = def.find_prop(key) {
            let value_hint = value_kind_hint(&prop.value);
            return make_hover(format!(
                "**`{key}`** — {}\n\n*Value:* {value_hint}",
                prop.description,
            ));
        }
    }

    make_hover(format!("`{key}` — unknown property on `{parent_name}`"))
}

fn hover_for_literal(
    lit: &str,
    prop_key: &str,
    parent_name: &str,
    aliases: &[AliasDeclaration],
) -> Hover {
    let real_name = resolve_real_name(parent_name, aliases);

    if let Some(def) = schema::lookup(real_name) {
        if let Some(prop) = def.find_prop(prop_key) {
            if let schema::ValueKind::Enum(variants) = &prop.value {
                let valid = variants.contains(&lit);
                let all = variants.join("`, `");
                if valid {
                    return make_hover(format!("`{lit}` ✓\n\nAllowed values: `{all}`"));
                } else {
                    return make_hover(format!(
                        "`{lit}` ✗ — not a valid value\n\nAllowed: `{all}`"
                    ));
                }
            }
        }
    }

    make_hover(format!("`{lit}`"))
}

/// Follow an alias chain to find the underlying built-in type name
fn resolve_real_name<'a>(name: &'a str, aliases: &'a [AliasDeclaration]) -> &'a str {
    let mut current = name;

    // guard against circular aliases (max 16 hops)
    for _ in 0..16 {
        if schema::lookup(current).is_some() {
            return current;
        }
        if let Some(alias) = aliases.iter().find(|a| a.name == current) {
            current = &alias.target.name;
        } else {
            return current;
        }
    }

    current
}

fn value_kind_hint(kind: &schema::ValueKind) -> String {
    match kind {
        schema::ValueKind::Enum(variants) => {
            format!("`{}` (enum)", variants.join("` | `"))
        }
        schema::ValueKind::Type(name) => format!("`{name}(...)`"),
        schema::ValueKind::UInt => "unsigned integer".to_string(),
        schema::ValueKind::Literal => "literal string".to_string(),
    }
}

fn span_contains(span: &Span, offset: usize) -> bool {
    offset >= span.start && offset < span.end
}

fn make_hover(text: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text,
        }),
        range: None,
    }
}
