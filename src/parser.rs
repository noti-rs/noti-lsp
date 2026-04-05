use crate::ast::*;
use pest::Parser;
use pest::error::Error as PestError;
use pest_derive::Parser as PestDerive;

#[derive(PestDerive)]
#[grammar = "src/layout.pest"]
struct LayoutParser;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl ParseError {
    fn from_pest(err: PestError<Rule>) -> Self {
        let offset = match err.location {
            pest::error::InputLocation::Pos(pos) => pos,
            pest::error::InputLocation::Span((span, _)) => span,
        };
        Self {
            message: err.to_string(),
            offset,
        }
    }
}

pub fn parse(src: &str) -> Result<Layout, ParseError> {
    let pairs = LayoutParser::parse(Rule::Layout, src).map_err(ParseError::from_pest)?;

    let layout_pair = pairs.into_iter().next().unwrap();

    let mut aliases = Vec::new();
    let mut root = None;

    for pair in layout_pair.into_inner() {
        match pair.as_rule() {
            Rule::AliasDefinitions => {
                for alias_pair in pair.into_inner() {
                    aliases.push(convert_alias(alias_pair));
                }
            }
            Rule::NodeType => {
                root = Some(convert_node(pair));
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Layout {
        aliases,
        root: root.expect("grammar guarantees a root NodeType"),
    })
}

fn convert_alias(pair: pest::iterators::Pair<Rule>) -> AliasDeclaration {
    let span = Span::from_pest(pair.as_span());
    let mut inner = pair.into_inner();

    inner.next();
    let ident_pair = inner.next().unwrap();
    let name = ident_pair.as_str().to_string();
    let name_span = Span::from_pest(ident_pair.as_span());
    inner.next();

    let target = convert_type_value(inner.next().unwrap());
    AliasDeclaration {
        name,
        name_span,
        target,
        span,
    }
}

fn convert_node(pair: pest::iterators::Pair<Rule>) -> NodeType {
    let span = Span::from_pest(pair.as_span());
    let mut inner = pair.into_inner();

    let ident_pair = inner.next().unwrap();
    let name = ident_pair.as_str().to_string();
    let name_span = Span::from_pest(ident_pair.as_span());

    let mut properties = Vec::new();
    let mut children = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::OpeningParenthesis
            | Rule::ClosingParenthesis
            | Rule::OpeningBrace
            | Rule::ClosingBrace => {}

            Rule::Properties => {
                properties = convert_properties(p);
            }
            Rule::Children => {
                for child in p.into_inner() {
                    if child.as_rule() == Rule::NodeType {
                        children.push(convert_node(child));
                    }
                }
            }
            _ => {}
        }
    }

    NodeType {
        name,
        name_span,
        properties,
        children,
        span,
    }
}

fn convert_properties(pair: pest::iterators::Pair<Rule>) -> Vec<Property> {
    pair.into_inner()
        .filter(|p| p.as_rule() == Rule::Property)
        .map(convert_property)
        .collect()
}

fn convert_property(pair: pest::iterators::Pair<Rule>) -> Property {
    let span = Span::from_pest(pair.as_span());
    let mut inner = pair.into_inner();

    let ident_pair = inner.next().unwrap();
    let key = ident_pair.as_str().to_string();
    let key_span = Span::from_pest(ident_pair.as_span());
    inner.next();

    let value = convert_property_value(inner.next().unwrap());
    Property {
        key,
        key_span,
        value,
        span,
    }
}

fn convert_property_value(pair: pest::iterators::Pair<Rule>) -> PropertyValue {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::TypeValue => PropertyValue::Type(convert_type_value(inner)),
        Rule::UInt => {
            let span = Span::from_pest(inner.as_span());
            let n = inner.as_str().parse().unwrap_or(0);
            PropertyValue::UInt(n, span)
        }
        Rule::Literal => {
            let span = Span::from_pest(inner.as_span());
            PropertyValue::Literal(inner.as_str().to_string(), span)
        }
        _ => unreachable!(),
    }
}

fn convert_type_value(pair: pest::iterators::Pair<Rule>) -> TypeValue {
    let span = Span::from_pest(pair.as_span());
    let mut inner = pair.into_inner();

    let ident_pair = inner.next().unwrap();
    let name = ident_pair.as_str().to_string();
    let name_span = Span::from_pest(ident_pair.as_span());

    let mut tv_inner = TypeValueInner::Empty;

    for p in inner {
        match p.as_rule() {
            Rule::OpeningParenthesis | Rule::ClosingParenthesis => {}

            Rule::Properties => {
                tv_inner = TypeValueInner::Properties(convert_properties(p));
            }
            Rule::PropertyValue => {
                tv_inner = TypeValueInner::Value(Box::new(convert_property_value(p)));
            }
            _ => {}
        }
    }

    TypeValue {
        name,
        name_span,
        inner: tv_inner,
        span,
    }
}
