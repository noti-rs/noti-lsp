#[derive(Debug, Clone)]
pub struct PropDef {
    pub name: &'static str,
    pub description: &'static str,
    pub value: ValueKind,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    Enum(&'static [&'static str]),
    Type(&'static str),
    UInt,
    Literal,
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: &'static str,
    pub description: &'static str,
    pub props: &'static [PropDef],
    pub constructor: Option<ValueKind>,
}

impl TypeDef {
    pub fn find_prop(&self, name: &str) -> Option<&PropDef> {
        self.props.iter().find(|p| p.name == name)
    }
}

pub fn lookup(name: &str) -> Option<&'static TypeDef> {
    TYPES.iter().find(|t| t.name == name)
}

pub static TYPES: &[TypeDef] = &[
    TypeDef {
        name: "FlexContainer",
        description: "A flexible container that lays out its children in a row or column.",
        constructor: None,
        props: &[
            PropDef {
                name: "direction",
                description: "The direction children are laid out.",
                value: ValueKind::Enum(&["horizontal", "vertical"]),
            },
            PropDef {
                name: "spacing",
                description: "Inner padding around the container's content.",
                value: ValueKind::Type("Spacing"),
            },
            PropDef {
                name: "alignment",
                description: "How children are aligned inside the container.",
                value: ValueKind::Type("Alignment"),
            },
            PropDef {
                name: "border",
                description: "Border drawn around the container.",
                value: ValueKind::Type("Border"),
            },
            PropDef {
                name: "max_width",
                description: "Maximum width in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "max_height",
                description: "Maximum height in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "min_width",
                description: "Minimum width in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "min_height",
                description: "Minimum height in pixels.",
                value: ValueKind::UInt,
            },
        ],
    },
    TypeDef {
        name: "Text",
        description: "Displays a text element from the notification payload.",
        constructor: None,
        props: &[
            PropDef {
                name: "kind",
                description: "Which text field from the notification to display.",
                value: ValueKind::Enum(&["title", "summary", "body"]),
            },
            PropDef {
                name: "style",
                description: "Font style applied to the text.",
                value: ValueKind::Enum(&["normal", "bold", "italic", "bold-italic"]),
            },
            PropDef {
                name: "justification",
                description: "Horizontal text alignment.",
                value: ValueKind::Enum(&["left", "center", "right", "fill"]),
            },
            PropDef {
                name: "wrap",
                description: "Whether the text wraps onto multiple lines.",
                value: ValueKind::Enum(&["true", "false"]),
            },
            PropDef {
                name: "ellipsize_at",
                description: "Where to truncate text that doesn't fit.",
                value: ValueKind::Enum(&["start", "middle", "end"]),
            },
            PropDef {
                name: "font_size",
                description: "Font size in points.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "line_spacing",
                description: "Extra spacing between lines in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "margin",
                description: "Outer margin around the text element.",
                value: ValueKind::Type("Spacing"),
            },
        ],
    },
    TypeDef {
        name: "Image",
        description: "Displays the notification icon or image.",
        constructor: None,
        props: &[
            PropDef {
                name: "max_size",
                description: "Maximum width and height of the image in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "max_width",
                description: "Maximum width of the image in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "max_height",
                description: "Maximum height of the image in pixels.",
                value: ValueKind::UInt,
            },
        ],
    },
    TypeDef {
        name: "Spacing",
        description: "Defines padding or margin on all four sides.",
        constructor: Some(ValueKind::UInt), // Spacing(10) sets all sides at once
        props: &[
            PropDef {
                name: "top",
                description: "Top spacing in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "right",
                description: "Right spacing in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "bottom",
                description: "Bottom spacing in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "left",
                description: "Left spacing in pixels.",
                value: ValueKind::UInt,
            },
        ],
    },
    TypeDef {
        name: "Alignment",
        description: "Controls how content is aligned on each axis.",
        constructor: None,
        props: &[
            PropDef {
                name: "horizontal",
                description: "Horizontal alignment of children.",
                value: ValueKind::Enum(&["start", "center", "end", "space_between"]),
            },
            PropDef {
                name: "vertical",
                description: "Vertical alignment of children.",
                value: ValueKind::Enum(&["start", "center", "end", "space_between"]),
            },
        ],
    },
    TypeDef {
        name: "Border",
        description: "Draws a border around a container.",
        constructor: None,
        props: &[
            PropDef {
                name: "size",
                description: "Border thickness in pixels.",
                value: ValueKind::UInt,
            },
            PropDef {
                name: "radius",
                description: "Corner radius in pixels.",
                value: ValueKind::UInt,
            },
        ],
    },
];
