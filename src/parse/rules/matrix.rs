use crate::ast::{AstNode, ParseResult};
use crate::error::RenderError;

use super::super::lexer::Parser;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "matrix" => Some(parse_matrix_command(parser)),
        _ => None,
    }
}

fn parse_matrix_command(parser: &mut Parser) -> ParseResult<AstNode> {
    let content = parser.consume_braced_content("matrix")?;
    let rows = parse_rows(&content)?;
    Ok(AstNode::Matrix(rows))
}

pub fn parse_rows(body: &str) -> ParseResult<Vec<Vec<AstNode>>> {
    let mut rows = Vec::new();
    for raw_row in body.split("\\\\") {
        let trimmed_row = raw_row.trim();
        if trimmed_row.is_empty() {
            continue;
        }
        let mut cells = Vec::new();
        for cell_str in trimmed_row.split('&') {
            let trimmed_cell = cell_str.trim();
            if trimmed_cell.is_empty() {
                cells.push(AstNode::Text(String::new()));
            } else {
                let mut nested = Parser::new(trimmed_cell);
                let cell_ast = nested.parse_group(None)?;
                cells.push(Parser::normalize_group_static(cell_ast));
            }
        }
        rows.push(cells);
    }
    if rows.is_empty() {
        Err(RenderError::ParseError("多行环境内容不能为空".into()))
    } else {
        Ok(rows)
    }
}
