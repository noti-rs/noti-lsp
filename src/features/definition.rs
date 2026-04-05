use crate::ast::{AliasDeclaration, Layout, NodeType, PropertyValue, Span, TypeValueInner};
use crate::document::Document;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

pub fn goto_definition(doc: &Document, pos: Position, uri: &Url) -> Option<GotoDefinitionResponse> {
    let offset = doc.position_to_offset(pos);
    let layout = doc.layout.as_ref()?;

    let alias_name = find_alias_usage_at(layout, offset)?;

    let decl = layout.aliases.iter().find(|a| a.name == alias_name)?;

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: uri.clone(),
        range: decl.name_span.to_range(doc),
    }))
}

fn find_alias_usage_at<'a>(layout: &'a Layout, offset: usize) -> Option<&'a str> {
    for alias in &layout.aliases {
        if alias.name_span.contains(offset) {
            return Some(&alias.name);
        }
        // Inside alias target (alias referencing another alias)
        if alias.target.span.contains(offset) {
            if let Some(name) = find_alias_in_type_value(&alias.target, layout, offset) {
                return Some(name);
            }
        }
    }

    find_alias_in_node(&layout.root, layout, offset)
}

fn find_alias_in_node<'a>(
    node: &'a NodeType,
    layout: &'a Layout,
    offset: usize,
) -> Option<&'a str> {
    if !node.span.contains(offset) {
        return None;
    }

    if node.name_span.contains(offset) && layout.is_alias(&node.name) {
        return Some(&node.name);
    }

    for prop in &node.properties {
        if let PropertyValue::Type(tv) = &prop.value {
            if let Some(name) = find_alias_in_type_value(tv, layout, offset) {
                return Some(name);
            }
        }
    }

    for child in &node.children {
        if let Some(name) = find_alias_in_node(child, layout, offset) {
            return Some(name);
        }
    }

    None
}

fn find_alias_in_type_value<'a>(
    tv: &'a crate::ast::TypeValue,
    layout: &'a Layout,
    offset: usize,
) -> Option<&'a str> {
    if tv.name_span.contains(offset) && layout.is_alias(&tv.name) {
        return Some(&tv.name);
    }

    if let TypeValueInner::Properties(props) = &tv.inner {
        for prop in props {
            if let PropertyValue::Type(inner_tv) = &prop.value {
                if let Some(name) = find_alias_in_type_value(inner_tv, layout, offset) {
                    return Some(name);
                }
            }
        }
    }

    None
}
