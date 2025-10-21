use crate::ast::{AstNode, Delimiter, ParseResult};
use crate::error::RenderError;

use super::super::lexer::Parser;
use super::{build_large_operator, handle_command, handle_text_command, is_large_operator};

pub fn handle(parser: &mut Parser, command: &str) -> Option<ParseResult<AstNode>> {
    match command {
        "left" => Some(parser.parse_delimited_expression()),
        _ => None,
    }
}

impl Parser {
    pub(crate) fn parse_delimited_expression(&mut self) -> ParseResult<AstNode> {
        let left = parse_delimiter_token(self)?;
        let mut nodes = Vec::new();
        loop {
            let ch = self
                .peek_char()
                .ok_or_else(|| RenderError::ParseError("缺少与 \\left 对应的 \\right".into()))?;
            match ch {
                '{' => {
                    self.consume_char();
                    let inner = self.parse_group(Some('}'))?;
                    nodes.push(inner);
                }
                '}' => {
                    return Err(RenderError::ParseError("检测到不成对的大括号".into()));
                }
                '^' | '_' => {
                    self.consume_char();
                    let script = self.parse_atom()?;
                    Parser::attach_script(&mut nodes, ch, script)?;
                }
                '\\' => {
                    self.consume_char();
                    let command = self.parse_command();
                    if command == "right" {
                        let right = parse_delimiter_token(self)?;
                        let inner = Parser::normalize_group_static(AstNode::Group(
                            Parser::merge_text_nodes(nodes),
                        ));
                        return Ok(AstNode::Delimited {
                            left,
                            inner: Box::new(inner),
                            right,
                        });
                    }
                    if let Some(result) = handle_command(self, &command) {
                        nodes.push(result?);
                    } else if let Some(mapped) = handle_text_command(&command) {
                        nodes.push(AstNode::Text(mapped.to_string()));
                    } else if is_large_operator(&command) {
                        nodes.push(build_large_operator(&command));
                    } else {
                        nodes.push(AstNode::Text(format!("\\{}", command)));
                    }
                }
                _ => {
                    let text = self.parse_text_segment();
                    if !text.is_empty() {
                        nodes.push(AstNode::Text(text));
                    }
                }
            }
        }
    }
}

fn parse_delimiter_token(parser: &mut Parser) -> ParseResult<Delimiter> {
    match parser.peek_char() {
        Some('.') => {
            parser.consume_char();
            Ok(Delimiter { glyph: None })
        }
        Some('\\') => {
            parser.consume_char();
            let name = parser.parse_command();
            let glyph = delimiter_command_to_glyph(&name).map(|s| s.map(|g| g.to_string()));
            match glyph {
                Some(value) => Ok(Delimiter { glyph: value }),
                None => Err(RenderError::ParseError(format!(
                    "未知的定界符命令 \\{}",
                    name
                ))),
            }
        }
        Some(ch) => {
            parser.consume_char();
            Ok(Delimiter {
                glyph: Some(ch.to_string()),
            })
        }
        None => Err(RenderError::ParseError("缺少定界符".into())),
    }
}

fn delimiter_command_to_glyph(name: &str) -> Option<Option<&'static str>> {
    match name {
        "langle" => Some(Some("⟨")),
        "rangle" => Some(Some("⟩")),
        "lceil" => Some(Some("⌈")),
        "rceil" => Some(Some("⌉")),
        "lfloor" => Some(Some("⌊")),
        "rfloor" => Some(Some("⌋")),
        "lbrace" => Some(Some("{")),
        "rbrace" => Some(Some("}")),
        "lvert" | "vert" | "rvert" => Some(Some("|")),
        "lVert" | "Vert" | "rVert" => Some(Some("‖")),
        "." => Some(None),
        _ => None,
    }
}
