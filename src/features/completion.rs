use crate::ast::{Layout, NodeType, PropertyValue, TypeValueInner};
use crate::document::Document;
use crate::schema::{self, ValueKind};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Documentation, InsertTextFormat,
    MarkupContent, MarkupKind, Position, Range, TextEdit,
};

pub fn completion_trigger_chars() -> Vec<String> {
    vec!['('.into(), ','.into(), ' '.into(), '='.into()]
}
pub fn get_completions(doc: &Document, pos: Position) -> Vec<CompletionItem> {
    let offset = doc.position_to_offset(pos);
    let Some(layout) = doc.layout.as_ref() else {
        return vec![];
    };

    if let Some(ctx) = find_context(layout, offset) {
        build_completions(ctx, layout, doc, pos)
    } else {
        vec![]
    }
}

#[derive(Debug)]
enum Context<'a> {
    /// Cursor is on/after a component name — complete type names
    TypeName,
    /// Cursor is inside a property list — complete prop keys
    /// We know the parent type name so we can filter props
    PropKey { parent: &'a str },
    /// Cursor is on the RHS of `key =` — complete values for that prop
    PropValue { parent: &'a str, key: &'a str },
}

fn find_context<'a>(layout: &'a Layout, offset: usize) -> Option<Context<'a>> {
    for alias in &layout.aliases {
        if alias.target.span.contains(offset) {
            if let Some(ctx) =
                context_in_type_value_inner(&alias.target.name, &alias.target.inner, offset)
            {
                return Some(ctx);
            }

            if alias.target.name_span.contains(offset) {
                return Some(Context::TypeName);
            }
        }
    }

    context_in_node(&layout.root, offset)
}

fn context_in_node<'a>(node: &'a NodeType, offset: usize) -> Option<Context<'a>> {
    if !node.span.contains(offset) {
        return None;
    }

    for child in &node.children {
        if child.span.contains(offset) {
            return context_in_node(child, offset);
        }
    }

    for prop in &node.properties {
        if prop.span.contains(offset) {
            if prop.key_span.contains(offset) {
                return Some(Context::PropKey { parent: &node.name });
            }

            return match &prop.value {
                PropertyValue::Type(tv) => {
                    // Recurse into nested type value
                    if let Some(ctx) = context_in_type_value_inner(&tv.name, &tv.inner, offset) {
                        return Some(ctx);
                    }
                    Some(Context::PropValue {
                        parent: &node.name,
                        key: &prop.key,
                    })
                }
                _ => Some(Context::PropValue {
                    parent: &node.name,
                    key: &prop.key,
                }),
            };
        }
    }

    // Inside the prop list but not on any existing prop
    // (empty list, or after a comma, or on a new line)
    Some(Context::PropKey { parent: &node.name })
}

fn context_in_type_value_inner<'a>(
    type_name: &'a str,
    inner: &'a TypeValueInner,
    offset: usize,
) -> Option<Context<'a>> {
    if let TypeValueInner::Properties(props) = inner {
        for prop in props {
            if prop.span.contains(offset) {
                if prop.key_span.contains(offset) {
                    return Some(Context::PropKey { parent: type_name });
                }
                return Some(Context::PropValue {
                    parent: type_name,
                    key: &prop.key,
                });
            }
        }

        // Inside the parens but not on a prop
        return Some(Context::PropKey { parent: type_name });
    }
    None
}

fn build_completions(
    ctx: Context,
    layout: &Layout,
    doc: &Document,
    pos: Position,
) -> Vec<CompletionItem> {
    match ctx {
        Context::TypeName => complete_type_names(layout),

        Context::PropKey { parent } => {
            let real = crate::utils::resolve_real_alias_name(parent, &layout.aliases);
            complete_prop_keys(real, layout)
        }

        Context::PropValue { parent, key } => {
            let real = crate::utils::resolve_real_alias_name(parent, &layout.aliases);
            complete_prop_value(real, key, layout, doc, pos)
        }
    }
}

fn complete_type_names(layout: &Layout) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = schema::TYPES
        .iter()
        .map(|t| CompletionItem {
            label: t.name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(t.description.to_string()),
            insert_text: Some(format!("{}(\n  $0\n)", t.name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect();

    // Also offer aliases
    for alias in &layout.aliases {
        items.push(CompletionItem {
            label: alias.name.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("alias for {}()", alias.target.name)),
            insert_text: Some(format!("{}(\n  $0\n)", alias.name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    items
}

fn complete_prop_keys(type_name: &str, layout: &Layout) -> Vec<CompletionItem> {
    let Some(def) = schema::lookup(type_name) else {
        return vec![];
    };

    def.props
        .iter()
        .map(|p| {
            // Snippet: insert `key = $0` or `key = TypeName(\n  $0\n)` for nested types
            let snippet = match &p.value {
                ValueKind::Type(t) => format!("{} = {}(\n  $0\n)", p.name, t),
                ValueKind::Enum(variants) => {
                    // Use a choice snippet: ${1|a,b,c|}
                    let choices = variants.join(",");
                    format!("{} = ${{1|{choices}|}}", p.name)
                }
                _ => format!("{} = $0", p.name),
            };

            CompletionItem {
                label: p.name.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(p.description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "{}\n\n*Value:* {}",
                        p.description,
                        crate::utils::value_kind_hint(&p.value)
                    ),
                })),
                insert_text: Some(snippet),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            }
        })
        .collect()
}

fn complete_prop_value(
    type_name: &str,
    key: &str,
    layout: &Layout,
    doc: &Document,
    pos: Position,
) -> Vec<CompletionItem> {
    let Some(def) = schema::lookup(type_name) else {
        return vec![];
    };
    let Some(prop) = def.find_prop(key) else {
        return vec![];
    };

    match &prop.value {
        ValueKind::Enum(variants) => variants
            .iter()
            .map(|v| CompletionItem {
                label: v.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: current_word_range(doc, pos),
                    new_text: v.to_string(),
                })),
                ..Default::default()
            })
            .collect(),

        ValueKind::Type(t) => vec![CompletionItem {
            label: t.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            insert_text: Some(format!("{t}(\n  $0\n)")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }],

        ValueKind::UInt => vec![CompletionItem {
            label: "0".to_string(),
            kind: Some(CompletionItemKind::VALUE),
            insert_text: Some("$0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }],

        ValueKind::Literal => vec![],
    }
}

fn current_word_range(doc: &Document, pos: Position) -> Range {
    let offset = doc.position_to_offset(pos);
    let src = doc.source.as_bytes();

    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';

    let mut start = offset;
    while start > 0 && is_word(src[start - 1]) {
        start -= 1;
    }

    let mut end = offset;
    while end < src.len() && is_word(src[end]) {
        end += 1;
    }

    Range {
        start: doc.offset_to_position(start),
        end: doc.offset_to_position(end),
    }
}
