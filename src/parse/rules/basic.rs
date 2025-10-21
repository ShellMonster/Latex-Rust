use crate::ast::{AstNode, ParseResult};

use super::super::lexer::Parser;

/// 处理基础 LaTeX 命令，例如 `\text{...}`
pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "text" => Some(handle_text_command(parser)),
        "displaystyle" => Some(Ok(AstNode::Group(Vec::new()))),
        _ => None,
    }
}

fn handle_text_command(parser: &mut Parser) -> ParseResult<AstNode> {
    let content = parser.consume_braced_content("text")?;
    Ok(AstNode::Text(content))
}
