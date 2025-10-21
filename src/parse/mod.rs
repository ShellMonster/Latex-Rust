mod lexer;
pub mod rules;

use crate::ast::{AstNode, ParseResult, ParsedFormula};
use crate::error::RenderError;

use lexer::Parser;

pub fn parse(input: &str) -> ParseResult<ParsedFormula> {
    if input.len() > 5 * 1024 {
        return Err(RenderError::ParseError("公式长度超过 5KB 限制".into()));
    }
    if !input.is_char_boundary(input.len()) {
        return Err(RenderError::InvalidUtf8);
    }

    let mut parser = Parser::new(input);
    let ast = parser.parse_group(None)?;
    Ok(ParsedFormula::new(parser.normalize_group(ast)))
}

impl Parser {
    pub(crate) fn parse_group(&mut self, stop: Option<char>) -> ParseResult<AstNode> {
        let mut nodes = Vec::with_capacity(16);
        while let Some(ch) = self.peek_char() {
            if let Some(end) = stop {
                if ch == end {
                    self.consume_char();
                    break;
                }
            }

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
                    Self::attach_script(&mut nodes, ch, script)?;
                }
                '\\' => {
                    self.consume_char();
                    let command = self.parse_command();
                    if let Some(result) = rules::handle_command(self, &command) {
                        nodes.push(result?);
                    } else if let Some(mapped) = rules::handle_text_command(&command) {
                        nodes.push(AstNode::Text(mapped.to_string()));
                    } else if rules::is_large_operator(&command) {
                        nodes.push(rules::build_large_operator(&command));
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
        Ok(AstNode::Group(Self::merge_text_nodes(nodes)))
    }

    pub(crate) fn parse_atom(&mut self) -> ParseResult<AstNode> {
        if let Some(ch) = self.peek_char() {
            match ch {
                '{' => {
                    self.consume_char();
                    self.parse_group(Some('}'))
                }
                '\\' => {
                    self.consume_char();
                    let command = self.parse_command();
                    if let Some(result) = rules::handle_command(self, &command) {
                        result
                    } else if let Some(mapped) = rules::handle_text_command(&command) {
                        Ok(AstNode::Text(mapped.to_string()))
                    } else if rules::is_large_operator(&command) {
                        Ok(rules::build_large_operator(&command))
                    } else {
                        Ok(AstNode::Text(format!("\\{}", command)))
                    }
                }
                _ => {
                    let ch = self.consume_char().unwrap();
                    Ok(AstNode::Text(ch.to_string()))
                }
            }
        } else {
            Err(RenderError::ParseError(
                "表达式意外结束，缺少上下标内容".into(),
            ))
        }
    }

    pub(crate) fn parse_block(&mut self, context: &str) -> ParseResult<AstNode> {
        let content = self.consume_braced_content(context)?;
        let mut nested = Parser::new(&content);
        let ast = nested.parse_group(None)?;
        Ok(Self::normalize_group_static(ast))
    }

    pub(crate) fn normalize_group(&self, node: AstNode) -> AstNode {
        Self::normalize_group_static(node)
    }

    pub(crate) fn normalize_group_static(node: AstNode) -> AstNode {
        match node {
            AstNode::Group(mut list) if list.len() == 1 => list.remove(0),
            other => other,
        }
    }

    pub(crate) fn merge_text_nodes(nodes: Vec<AstNode>) -> Vec<AstNode> {
        let mut merged = Vec::with_capacity(nodes.len());
        for node in nodes {
            match (&node, merged.last_mut()) {
                (AstNode::Text(current), Some(AstNode::Text(prev))) => prev.push_str(current),
                _ => merged.push(node),
            }
        }
        merged
    }

    pub(crate) fn attach_script(
        stack: &mut Vec<AstNode>,
        symbol: char,
        script: AstNode,
    ) -> ParseResult<()> {
        let base = stack
            .pop()
            .ok_or_else(|| RenderError::ParseError("上下标缺少前导元素".into()))?;

        let mut scripts = match base {
            AstNode::Scripts {
                base,
                superscript,
                subscript,
            } => AstNode::Scripts {
                base,
                superscript,
                subscript,
            },
            other => AstNode::Scripts {
                base: Box::new(other),
                superscript: None,
                subscript: None,
            },
        };

        match (&mut scripts, symbol) {
            (
                AstNode::Scripts {
                    superscript: target,
                    ..
                },
                '^',
            ) => {
                if target.is_some() {
                    return Err(RenderError::ParseError("重复设置上标".into()));
                }
                *target = Some(Box::new(script));
            }
            (
                AstNode::Scripts {
                    subscript: target, ..
                },
                '_',
            ) => {
                if target.is_some() {
                    return Err(RenderError::ParseError("重复设置下标".into()));
                }
                *target = Some(Box::new(script));
            }
            _ => {}
        }

        stack.push(scripts);
        Ok(())
    }
}
