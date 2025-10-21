use crate::ast::{AstNode, Delimiter, ParseResult};
use crate::error::RenderError;

use super::super::lexer::Parser;
use super::matrix;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    if command == "begin" {
        Some(parse_environment(parser))
    } else {
        None
    }
}

fn parse_environment(parser: &mut Parser) -> ParseResult<AstNode> {
    let name = parser.consume_braced_content("环境名称")?;
    let body = consume_environment_body(parser, &name)?;
    let rows = matrix::parse_rows(&body)?;

    match name.as_str() {
        "cases" => Ok(make_delimited("{", None, rows)),
        "aligned" | "align" | "array" => Ok(AstNode::Matrix(rows)),
        "pmatrix" => Ok(make_delimited("(", Some(")"), rows)),
        "bmatrix" => Ok(make_delimited("[", Some("]"), rows)),
        "Bmatrix" => Ok(make_delimited("{", Some("}"), rows)),
        "vmatrix" => Ok(make_delimited("|", Some("|"), rows)),
        "Vmatrix" => Ok(make_delimited("‖", Some("‖"), rows)),
        "matrix" => Ok(AstNode::Matrix(rows)),
        other => Err(RenderError::ParseError(format!("暂不支持环境 {other}"))),
    }
}

fn make_delimited(left: &str, right: Option<&str>, rows: Vec<Vec<AstNode>>) -> AstNode {
    AstNode::Delimited {
        left: Delimiter {
            glyph: Some(left.to_string()),
        },
        inner: Box::new(AstNode::Matrix(rows)),
        right: Delimiter {
            glyph: right.map(|g| g.to_string()),
        },
    }
}

fn consume_environment_body(parser: &mut Parser, name: &str) -> ParseResult<String> {
    let mut depth = 1;
    let mut closed = false;
    let mut body = String::new();

    while parser.peek_char().is_some() {
        if parser.starts_with_str("\\begin{") {
            parser.consume_char(); // '\'
            body.push('\\');
            parser.advance("begin".len());
            body.push_str("begin");
            let nested = parser.consume_braced_content("环境名称")?;
            body.push('{');
            body.push_str(&nested);
            body.push('}');
            if nested == name {
                depth += 1;
            }
            continue;
        }

        if parser.starts_with_str("\\end{") {
            parser.consume_char(); // '\'
            parser.advance("end".len());
            let env_name = parser.consume_braced_content("环境名称")?;
            if env_name == name {
                if depth == 1 {
                    closed = true;
                    break;
                } else {
                    depth -= 1;
                    body.push('\\');
                    body.push_str("end");
                    body.push('{');
                    body.push_str(&env_name);
                    body.push('}');
                    continue;
                }
            } else {
                body.push('\\');
                body.push_str("end");
                body.push('{');
                body.push_str(&env_name);
                body.push('}');
                continue;
            }
        }

        if let Some(ch) = parser.consume_char() {
            body.push(ch);
        } else {
            break;
        }
    }

    if !closed {
        return Err(RenderError::ParseError(format!(
            "环境 {name} 缺少匹配的 \\end{{{name}}}"
        )));
    }
    Ok(body)
}
