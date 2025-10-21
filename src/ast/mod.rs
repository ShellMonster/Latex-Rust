use crate::error::RenderError;

#[derive(Debug, Clone)]
pub enum AstNode {
    Text(String),
    Group(Vec<AstNode>),
    Fraction {
        numerator: Box<AstNode>,
        denominator: Box<AstNode>,
    },
    Sqrt {
        value: Box<AstNode>,
    },
    Delimited {
        left: Delimiter,
        inner: Box<AstNode>,
        right: Delimiter,
    },
    LargeOperator(LargeOperatorNode),
    Symbol(SpecialSymbol),
    Matrix(Vec<Vec<AstNode>>),
    Decorated {
        base: Box<AstNode>,
        decoration: DecorationKind,
    },
    Scripts {
        base: Box<AstNode>,
        superscript: Option<Box<AstNode>>,
        subscript: Option<Box<AstNode>>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum DecorationKind {
    Overline,
    Underline,
    Hat,
    Bar,
    Tilde,
    Vector,
    Dot,
    Ddot,
    Overbrace,
    Underbrace,
}

#[derive(Debug, Clone)]
pub struct LargeOperatorNode {
    pub content: String,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct Delimiter {
    pub glyph: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialSymbol {
    Sum,
    Product,
    Integral,
}

#[derive(Debug, Clone)]
pub struct ParsedFormula {
    pub ast: AstNode,
}

impl ParsedFormula {
    pub fn new(ast: AstNode) -> Self {
        Self { ast }
    }
}

pub type ParseResult<T> = Result<T, RenderError>;
