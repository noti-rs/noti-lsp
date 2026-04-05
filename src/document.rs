use crate::ast::Layout;
use crate::parser::{self, ParseError};
use tower_lsp::lsp_types::Position;

#[derive(Debug, Clone)]
pub struct Document {
    pub source: String,
    pub layout: Option<Layout>, // None when parse fails
    pub errors: Vec<ParseError>,
    pub lines: Vec<usize>,
}

impl Document {
    // pub fn new(source: String) -> Self {
    //     let lines = line_offsets(&source);
    //     let (layout, errors) = match parser::parse(&source) {
    //         Ok(l) => (Some(l), vec![]),
    //         Err(e) => (None, vec![e]),
    //     };
    //     Self {
    //         source,
    //         layout,
    //         errors,
    //         lines,
    //     }
    // }

    pub fn new(source: String) -> Self {
        // NOTE: temporary, since im working on windows rn
        let source = source.replace("\r\n", "\n");

        let lines = line_offsets(&source);
        let (layout, errors) = match parser::parse(&source) {
            Ok(l) => (Some(l), vec![]),
            Err(e) => (None, vec![e]),
        };
        Self {
            source,
            layout,
            errors,
            lines,
        }
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self
            .lines
            .partition_point(|&l| l <= offset)
            .saturating_sub(1);
        let col = offset - self.lines[line];
        Position {
            line: line as u32,
            character: col as u32,
        }
    }

    pub fn position_to_offset(&self, pos: Position) -> usize {
        let line = (pos.line as usize).min(self.lines.len() - 1);
        self.lines[line] + pos.character as usize
    }
}

fn line_offsets(src: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, ch) in src.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}
