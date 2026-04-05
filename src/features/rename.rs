use crate::ast::{AliasDeclaration, Layout, NodeType, PropertyValue, Span, TypeValueInner};
use crate::document::Document;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, PrepareRenameResponse, Range, TextEdit, WorkspaceEdit};

pub fn prepare_rename(doc: &Document, pos: Position) -> Option<PrepareRenameResponse> {
    let offset = doc.position_to_offset(pos);
    let layout = doc.layout.as_ref()?;

    find_alias_name_at(layout, offset).map(|(name, span)| {
        PrepareRenameResponse::RangeWithPlaceholder {
            range: span.to_range(doc),
            placeholder: name.to_string(),
        }
    })
}

pub fn rename(
    doc: &Document,
    pos: Position,
    new_name: String,
    uri: String,
) -> Option<WorkspaceEdit> {
    let offset = doc.position_to_offset(pos);
    let layout = doc.layout.as_ref()?;

    let (old_name, _) = find_alias_name_at(layout, offset)?;
    let old_name = old_name.to_string();

    let mut edits: Vec<TextEdit> = vec![];

    // Rename the declaration
    for alias in &layout.aliases {
        if alias.name == old_name {
            edits.push(TextEdit {
                range: alias.name_span.to_range(doc),
                new_text: new_name.clone(),
            });
        }
    }

    // Rename all usage sites
    collect_usages_in_node(&layout.root, &old_name, &new_name, doc, &mut edits);

    // Also rename usages inside alias targets
    for alias in &layout.aliases {
        collect_usages_in_type_value_name(&alias.target, &old_name, &new_name, doc, &mut edits);
    }

    if edits.is_empty() {
        return None;
    }

    let mut changes = HashMap::new();
    changes.insert(tower_lsp::lsp_types::Url::parse(&uri).ok()?, edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}

fn find_alias_name_at<'a>(layout: &'a Layout, offset: usize) -> Option<(&'a str, &'a Span)> {
    for alias in &layout.aliases {
        if alias.name_span.contains(offset) {
            return Some((&alias.name, &alias.name_span));
        }

        if let Some(result) = find_alias_usage_in_type_value(&alias.target, layout, offset) {
            return Some(result);
        }
    }

    find_alias_usage_in_node(&layout.root, layout, offset)
}

fn find_alias_usage_in_node<'a>(
    node: &'a NodeType,
    layout: &'a Layout,
    offset: usize,
) -> Option<(&'a str, &'a Span)> {
    if !node.span.contains(offset) {
        return None;
    }

    // The node name itself — only if it's an alias
    if node.name_span.contains(offset) {
        if layout.is_alias(&node.name) {
            return Some((&node.name, &node.name_span));
        } else {
            // It's a built-in — not renameable
            return None;
        }
    }

    // Inside property values
    for prop in &node.properties {
        if let PropertyValue::Type(tv) = &prop.value {
            if let Some(r) = find_alias_usage_in_type_value(tv, layout, offset) {
                return Some(r);
            }
        }
    }

    // Recurse into children
    for child in &node.children {
        if let Some(r) = find_alias_usage_in_node(child, layout, offset) {
            return Some(r);
        }
    }

    None
}

fn find_alias_usage_in_type_value<'a>(
    tv: &'a crate::ast::TypeValue,
    layout: &'a Layout,
    offset: usize,
) -> Option<(&'a str, &'a Span)> {
    if tv.name_span.contains(offset) && layout.is_alias(&tv.name) {
        return Some((&tv.name, &tv.name_span));
    }

    if let TypeValueInner::Properties(props) = &tv.inner {
        for prop in props {
            if let PropertyValue::Type(inner_tv) = &prop.value {
                if let Some(r) = find_alias_usage_in_type_value(inner_tv, layout, offset) {
                    return Some(r);
                }
            }
        }
    }

    None
}

fn collect_usages_in_node(
    node: &NodeType,
    old_name: &str,
    new_name: &str,
    doc: &Document,
    edits: &mut Vec<TextEdit>,
) {
    if node.name == old_name {
        edits.push(TextEdit {
            range: node.name_span.to_range(doc),
            new_text: new_name.to_string(),
        });
    }

    for prop in &node.properties {
        if let PropertyValue::Type(tv) = &prop.value {
            collect_usages_in_type_value_name(tv, old_name, new_name, doc, edits);
        }
    }

    for child in &node.children {
        collect_usages_in_node(child, old_name, new_name, doc, edits);
    }
}

fn collect_usages_in_type_value_name(
    tv: &crate::ast::TypeValue,
    old_name: &str,
    new_name: &str,
    doc: &Document,
    edits: &mut Vec<TextEdit>,
) {
    if tv.name == old_name {
        edits.push(TextEdit {
            range: tv.name_span.to_range(doc),
            new_text: new_name.to_string(),
        });
    }

    if let TypeValueInner::Properties(props) = &tv.inner {
        for prop in props {
            if let PropertyValue::Type(inner_tv) = &prop.value {
                collect_usages_in_type_value_name(inner_tv, old_name, new_name, doc, edits);
            }
        }
    }
}
