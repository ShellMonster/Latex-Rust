use crate::ast::ParseResult;
use crate::error::RenderError;

pub struct Parser {
    source: Vec<char>,
    len: usize,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let len = chars.len();
        Self {
            source: chars,
            len,
            pos: 0,
        }
    }

    #[inline]
    pub(crate) fn peek_char(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    #[inline]
    pub(crate) fn consume_char(&mut self) -> Option<char> {
        if self.pos < self.len {
            let ch = unsafe { *self.source.get_unchecked(self.pos) };
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn advance(&mut self, count: usize) {
        self.pos = (self.pos + count).min(self.len);
    }

    pub(crate) fn parse_command(&mut self) -> String {
        let mut name = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphabetic() {
                name.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        if name.is_empty() {
            if let Some(ch) = self.consume_char() {
                name.push(ch);
            }
        }
        name
    }

    pub(crate) fn parse_text_segment(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if matches!(ch, '{' | '}' | '^' | '_' | '\\') {
                break;
            }
            self.pos += 1;
        }
        self.source[start..self.pos].iter().collect()
    }

    pub(crate) fn consume_braced_content(&mut self, context: &str) -> ParseResult<String> {
        match self.peek_char() {
            Some('{') => {
                self.pos += 1;
                let mut depth = 1;
                let mut content = String::new();
                while self.pos < self.len {
                    let ch = unsafe { *self.source.get_unchecked(self.pos) };
                    self.pos += 1;
                    match ch {
                        '{' => {
                            depth += 1;
                            content.push(ch);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Ok(content);
                            } else {
                                content.push(ch);
                            }
                        }
                        _ => content.push(ch),
                    }
                }
                Err(RenderError::ParseError(format!(
                    "{context} 缺少匹配的大括号"
                )))
            }
            _ => Err(RenderError::ParseError(format!(
                "{context} 需要使用 {{...}} 包裹"
            ))),
        }
    }

    pub(crate) fn starts_with_str(&self, pattern: &str) -> bool {
        let mut idx = self.pos;
        for ch in pattern.chars() {
            if idx >= self.len || unsafe { *self.source.get_unchecked(idx) } != ch {
                return false;
            }
            idx += 1;
        }
        true
    }
}
