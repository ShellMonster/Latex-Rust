use super::super::lexer::Parser;
use crate::ast::{AstNode, LargeOperatorNode, ParseResult, SpecialSymbol};
use phf::phf_map;

static LARGE_OPERATORS: phf::Map<&'static str, (f32, &'static str)> = phf_map! {
    "lim" => (1.1, "lim"),
    "limsup" => (1.0, "limsup"),
    "liminf" => (1.0, "liminf"),
    "max" => (1.1, "max"),
    "min" => (1.1, "min"),
    "sup" => (1.1, "sup"),
    "inf" => (1.1, "inf"),
    "argmax" => (1.0, "argmax"),
    "argmin" => (1.0, "argmin"),
    "bigcup" => (1.1, "⋃"),
    "bigcap" => (1.1, "⋂"),
    "bigsqcup" => (1.1, "⨆"),
    "bigvee" => (1.1, "⋁"),
    "bigwedge" => (1.1, "⋀"),
    "bigoplus" => (1.1, "⨁"),
    "bigotimes" => (1.1, "⨂"),
    "bigodot" => (1.1, "⨀"),
    "coprod" => (1.1, "∐"),
};

pub fn handle(_parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "sum" => Some(Ok(AstNode::Symbol(SpecialSymbol::Sum))),
        "prod" => Some(Ok(AstNode::Symbol(SpecialSymbol::Product))),
        "int" | "oint" => Some(Ok(AstNode::Symbol(SpecialSymbol::Integral))),
        name if is_large_operator(name) => Some(Ok(build_large_operator(name))),
        _ => None,
    }
}

static OP_FUNCTIONS: phf::Map<&'static str, &'static str> = phf_map! {
    "log" => "log",
    "ln" => "ln",
    "exp" => "exp",
    "det" => "det",
    "sup" => "sup",
    "inf" => "inf",
    "dim" => "dim",
    "ker" => "ker",
};

pub fn map_function_name(command: &str) -> Option<&'static str> {
    OP_FUNCTIONS.get(command).copied()
}

pub fn is_large_operator(command: &str) -> bool {
    LARGE_OPERATORS.contains_key(command)
}

pub fn build_large_operator(command: &str) -> AstNode {
    let (scale, display) = LARGE_OPERATORS
        .get(command)
        .copied()
        .unwrap_or((1.0, command));
    AstNode::LargeOperator(LargeOperatorNode {
        content: display.to_string(),
        scale,
    })
}
