mod basic;
mod decorations;
mod delimiters;
mod environments;
mod fractions;
mod functions;
mod matrix;
mod operators;
mod roots;
mod spacing;
mod styles;
mod symbols;

use crate::ast::{AstNode, ParseResult};

use super::lexer::Parser;

pub fn handle_command(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    basic::handle(parser, command)
        .or_else(|| environments::handle(parser, command))
        .or_else(|| fractions::handle(parser, command))
        .or_else(|| roots::handle(parser, command))
        .or_else(|| delimiters::handle(parser, command))
        .or_else(|| matrix::handle(parser, command))
        .or_else(|| decorations::handle(parser, command))
        .or_else(|| styles::handle(parser, command))
        .or_else(|| operators::handle(parser, command))
}

pub fn handle_text_command(command: &str) -> Option<&'static str> {
    functions::map_text_command(command)
        .or_else(|| operators::map_function_name(command))
        .or_else(|| symbols::map_symbol(command))
        .or_else(|| spacing::map_spacing(command))
}

pub fn is_large_operator(command: &str) -> bool {
    operators::is_large_operator(command)
}

pub fn build_large_operator(command: &str) -> AstNode {
    operators::build_large_operator(command)
}
