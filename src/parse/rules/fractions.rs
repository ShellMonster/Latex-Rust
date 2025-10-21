use crate::ast::{AstNode, Delimiter, ParseResult};

use super::super::lexer::Parser;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "frac" => Some(handle_frac(parser)),
        "binom" => Some(handle_binom(parser)),
        _ => None,
    }
}

fn handle_frac(parser: &mut Parser) -> ParseResult<AstNode> {
    let numerator = parser.parse_block("分子")?;
    let denominator = parser.parse_block("分母")?;
    Ok(AstNode::Fraction {
        numerator: Box::new(numerator),
        denominator: Box::new(denominator),
    })
}

fn handle_binom(parser: &mut Parser) -> ParseResult<AstNode> {
    let top = parser.parse_block("binom 上部分")?;
    let bottom = parser.parse_block("binom 下部分")?;
    let matrix = AstNode::Matrix(vec![vec![top], vec![bottom]]);
    Ok(AstNode::Delimited {
        left: Delimiter {
            glyph: Some("(".to_string()),
        },
        inner: Box::new(matrix),
        right: Delimiter {
            glyph: Some(")".to_string()),
        },
    })
}
