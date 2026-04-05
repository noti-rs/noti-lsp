#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn from_pest(span: pest::Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AliasDeclaration {
    pub name: String,
    pub name_span: Span,
    pub target: TypeValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NodeType {
    pub name: String,
    pub name_span: Span,
    pub properties: Vec<Property>,
    pub children: Vec<NodeType>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub key_span: Span,
    pub value: PropertyValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Type(TypeValue),
    UInt(u64, Span),
    Literal(String, Span),
}

#[derive(Debug, Clone)]
pub struct TypeValue {
    pub name: String,
    pub name_span: Span,
    pub inner: TypeValueInner,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeValueInner {
    Properties(Vec<Property>),
    Value(Box<PropertyValue>),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub aliases: Vec<AliasDeclaration>,
    pub root: NodeType,
}
