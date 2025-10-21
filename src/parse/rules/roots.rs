use crate::ast::{AstNode, ParseResult};
use crate::error::RenderError;

use super::super::lexer::Parser;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "sqrt" => Some(handle_sqrt(parser)),
        _ => None,
    }
}

fn handle_sqrt(parser: &mut Parser) -> ParseResult<AstNode> {
    let index = parse_optional_index(parser)?;
    let value = parser.parse_block("根号内部")?;
    let base = AstNode::Sqrt {
        value: Box::new(value),
    };
    if let Some(index_node) = index {
        Ok(AstNode::Scripts {
            base: Box::new(base),
            superscript: Some(Box::new(index_node)),
            subscript: None,
        })
    } else {
        Ok(base)
    }
}

fn parse_optional_index(parser: &mut Parser) -> ParseResult<Option<AstNode>> {
    if parser.peek_char() != Some('[') {
        return Ok(None);
    }
    parser.consume_char();
    let mut depth = 0;
    let mut content = String::new();
    let mut found_closing = false;
    while let Some(ch) = parser.consume_char() {
        match ch {
            '[' => {
                depth += 1;
                content.push(ch);
            }
            ']' => {
                if depth == 0 {
                    found_closing = true;
                    break;
                } else {
                    depth -= 1;
                    content.push(ch);
                }
            }
            _ => content.push(ch),
        }
    }
    if !found_closing || depth != 0 {
        return Err(RenderError::ParseError("根号指数缺少匹配的方括号".into()));
    }
    if content.trim().is_empty() {
        return Ok(None);
    }
    let mut nested = Parser::new(content.trim());
    let ast = nested.parse_group(None)?;
    Ok(Some(Parser::normalize_group_static(ast)))
}
