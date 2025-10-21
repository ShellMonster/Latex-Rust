use crate::ast::{AstNode, ParseResult};

use super::super::lexer::Parser;

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    let style = match command {
        "mathbf" => Some(TextStyle::Bold),
        "mathit" => Some(TextStyle::Italic),
        "mathrm" => Some(TextStyle::Roman),
        "mathsf" => Some(TextStyle::SansSerif),
        "mathtt" => Some(TextStyle::Monospace),
        "mathbb" => Some(TextStyle::DoubleStruck),
        "mathcal" => Some(TextStyle::Calligraphic),
        "mathfrak" => Some(TextStyle::Fraktur),
        _ => None,
    }?;

    Some(handle_style(parser, style))
}

fn handle_style(parser: &mut Parser, style: TextStyle) -> ParseResult<AstNode> {
    let base = parser.parse_block("样式表达式")?;
    Ok(apply_style(base, style))
}

fn apply_style(node: AstNode, style: TextStyle) -> AstNode {
    match node {
        AstNode::Text(content) => AstNode::Text(apply_style_to_text(&content, style)),
        AstNode::Group(children) => AstNode::Group(
            children
                .into_iter()
                .map(|child| apply_style(child, style))
                .collect(),
        ),
        AstNode::Fraction {
            numerator,
            denominator,
        } => AstNode::Fraction {
            numerator: Box::new(apply_style(*numerator, style)),
            denominator: Box::new(apply_style(*denominator, style)),
        },
        AstNode::Sqrt { value } => AstNode::Sqrt {
            value: Box::new(apply_style(*value, style)),
        },
        AstNode::Delimited { left, inner, right } => AstNode::Delimited {
            left,
            inner: Box::new(apply_style(*inner, style)),
            right,
        },
        AstNode::LargeOperator(op) => AstNode::LargeOperator(op),
        AstNode::Symbol(sym) => AstNode::Symbol(sym),
        AstNode::Matrix(rows) => AstNode::Matrix(
            rows.into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|cell| apply_style(cell, style))
                        .collect()
                })
                .collect(),
        ),
        AstNode::Decorated { base, decoration } => AstNode::Decorated {
            base: Box::new(apply_style(*base, style)),
            decoration,
        },
        AstNode::Scripts {
            base,
            superscript,
            subscript,
        } => AstNode::Scripts {
            base: Box::new(apply_style(*base, style)),
            superscript: superscript.map(|node| Box::new(apply_style(*node, style))),
            subscript: subscript.map(|node| Box::new(apply_style(*node, style))),
        },
    }
}

#[derive(Copy, Clone)]
enum TextStyle {
    Bold,
    Italic,
    Roman,
    SansSerif,
    Monospace,
    DoubleStruck,
    Calligraphic,
    Fraktur,
}

fn apply_style_to_text(content: &str, style: TextStyle) -> String {
    content
        .chars()
        .map(|ch| map_char(ch, style).unwrap_or(ch))
        .collect()
}

fn map_char(ch: char, style: TextStyle) -> Option<char> {
    match style {
        TextStyle::Bold => map_bold(ch),
        TextStyle::Italic => None,    // 默认即为斜体
        TextStyle::Roman => Some(ch), // 先维持原字形
        TextStyle::SansSerif => map_sans_serif(ch),
        TextStyle::Monospace => map_monospace(ch),
        TextStyle::DoubleStruck => map_double_struck(ch),
        TextStyle::Calligraphic => map_calligraphic(ch),
        TextStyle::Fraktur => map_fraktur(ch),
    }
}

fn map_bold(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => Some(char::from_u32(0x1D400 + (ch as u32 - 'A' as u32))?),
        'a'..='z' => Some(char::from_u32(0x1D41A + (ch as u32 - 'a' as u32))?),
        '0'..='9' => Some(char::from_u32(0x1D7CE + (ch as u32 - '0' as u32))?),
        _ => None,
    }
}

fn map_sans_serif(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => Some(char::from_u32(0x1D5A0 + (ch as u32 - 'A' as u32))?),
        'a'..='z' => Some(char::from_u32(0x1D5BA + (ch as u32 - 'a' as u32))?),
        '0'..='9' => Some(char::from_u32(0x1D7E2 + (ch as u32 - '0' as u32))?),
        _ => None,
    }
}

fn map_monospace(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => Some(char::from_u32(0x1D670 + (ch as u32 - 'A' as u32))?),
        'a'..='z' => Some(char::from_u32(0x1D68A + (ch as u32 - 'a' as u32))?),
        '0'..='9' => Some(char::from_u32(0x1D7F6 + (ch as u32 - '0' as u32))?),
        _ => None,
    }
}

fn map_double_struck(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => Some(char::from_u32(0x1D538 + (ch as u32 - 'A' as u32))?),
        'a'..='z' => Some(char::from_u32(0x1D552 + (ch as u32 - 'a' as u32))?),
        '0'..='9' => Some(char::from_u32(0x1D7D8 + (ch as u32 - '0' as u32))?),
        _ => None,
    }
}

fn map_calligraphic(ch: char) -> Option<char> {
    const TABLE: &[Option<char>] = &[
        Some('\u{1D49C}'),
        Some('\u{212C}'),
        Some('\u{1D49E}'),
        Some('\u{1D49F}'),
        Some('\u{2130}'),
        Some('\u{2131}'),
        Some('\u{1D4A2}'),
        Some('\u{210B}'),
        Some('\u{2110}'),
        Some('\u{1D4A5}'),
        Some('\u{1D4A6}'),
        Some('\u{2112}'),
        Some('\u{2133}'),
        Some('\u{1D4A9}'),
        Some('\u{1D4AA}'),
        Some('\u{1D4AB}'),
        Some('\u{1D4AC}'),
        Some('\u{211B}'),
        Some('\u{1D4AE}'),
        Some('\u{1D4AF}'),
        Some('\u{1D4B0}'),
        Some('\u{1D4B1}'),
        Some('\u{1D4B2}'),
        Some('\u{1D4B3}'),
        Some('\u{1D4B4}'),
        Some('\u{1D4B5}'),
    ];
    match ch {
        'A'..='Z' => TABLE[ch as usize - 'A' as usize],
        _ => None,
    }
}

fn map_fraktur(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => Some(char::from_u32(0x1D504 + (ch as u32 - 'A' as u32))?),
        'a'..='z' => Some(char::from_u32(0x1D51E + (ch as u32 - 'a' as u32))?),
        _ => None,
    }
}
