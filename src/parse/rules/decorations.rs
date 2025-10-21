use crate::ast::{AstNode, DecorationKind, ParseResult};

use super::super::lexer::Parser;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "xrightarrow" => return Some(handle_stacked_arrow(parser, "→")),
        "xleftarrow" => return Some(handle_stacked_arrow(parser, "←")),
        "xRightarrow" => return Some(handle_stacked_arrow(parser, "⇒")),
        "xLeftarrow" => return Some(handle_stacked_arrow(parser, "⇐")),
        "xLeftrightarrow" => return Some(handle_stacked_arrow(parser, "⇔")),
        _ => {}
    }

    let decoration = match command {
        "overline" => Some(DecorationKind::Overline),
        "underline" => Some(DecorationKind::Underline),
        "hat" => Some(DecorationKind::Hat),
        "bar" => Some(DecorationKind::Bar),
        "tilde" => Some(DecorationKind::Tilde),
        "vec" => Some(DecorationKind::Vector),
        "dot" => Some(DecorationKind::Dot),
        "ddot" => Some(DecorationKind::Ddot),
        "overbrace" => Some(DecorationKind::Overbrace),
        "underbrace" => Some(DecorationKind::Underbrace),
        _ => None,
    }?;

    Some(handle_decorated(parser, decoration))
}

fn handle_decorated(parser: &mut Parser, decoration: DecorationKind) -> ParseResult<AstNode> {
    let base = parser.parse_block("装饰表达式")?;
    Ok(AstNode::Decorated {
        base: Box::new(base),
        decoration,
    })
}

fn handle_stacked_arrow(parser: &mut Parser, arrow: &str) -> ParseResult<AstNode> {
    let label = parser.parse_block("箭头标签")?;
    Ok(AstNode::Scripts {
        base: Box::new(AstNode::Text(arrow.to_string())),
        superscript: Some(Box::new(label)),
        subscript: None,
    })
}
