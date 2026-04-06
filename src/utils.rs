use crate::ast::AliasDeclaration;
use crate::schema::ValueKind;

pub fn resolve_real_alias_name<'a>(name: &'a str, aliases: &'a [AliasDeclaration]) -> &'a str {
    let mut current = name;

    // guard against circular aliases (max 16 hops)
    for _ in 0..16 {
        if crate::schema::lookup(current).is_some() {
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

pub fn value_kind_hint(kind: &ValueKind) -> String {
    match kind {
        ValueKind::Enum(variants) => format!("`{}`", variants.join("` | `")),
        ValueKind::Type(name) => format!("`{name}(...)`"),
        ValueKind::UInt => "unsigned integer".to_string(),
        ValueKind::Literal => "literal string".to_string(),
    }
}
