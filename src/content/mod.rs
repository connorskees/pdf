use std::borrow::{Borrow, Cow};

use crate::{
    error::PdfResult,
    lex::{LexBase, LexObject},
    objects::Object,
};

use operator::Operator;

mod operator;

pub struct ContentLexer<'a> {
    buffer: Cow<'a, [u8]>,
    cursor: usize,

    /// If >0, unrecognized operators will be ignored
    ///
    /// Set to true when encountering a `BX` operator, and to false
    /// when an `EX` operator is encountered
    in_compatibility_mode: u128,
}

#[derive(Debug)]
pub enum ContentToken {
    Object(Object),
    Operator(Operator),
}

#[derive(Debug)]
enum ContentTokenOrUnknownOperator {
    Token(ContentToken),
    UnknownOperator(String),
}

impl<'a> Iterator for ContentLexer<'a> {
    type Item = PdfResult<ContentToken>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Some(Ok(ContentTokenOrUnknownOperator::Token(tok))) => Some(Ok(tok)),
            Some(Ok(ContentTokenOrUnknownOperator::UnknownOperator(s))) => {
                if self.in_compatibility_mode() {
                    self.next()
                } else {
                    todo!("Unknown operator: {:?}", s)
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

impl<'a> ContentLexer<'a> {
    pub fn new(buffer: Cow<'a, [u8]>) -> Self {
        Self {
            buffer,
            cursor: 0,
            in_compatibility_mode: 0,
        }
    }

    fn enter_compatibility_mode(&mut self) {
        self.in_compatibility_mode <<= 1;
        self.in_compatibility_mode |= 1;
    }

    fn exit_compatibility_mode(&mut self) {
        self.in_compatibility_mode >>= 1;
    }

    fn in_compatibility_mode(&self) -> bool {
        self.in_compatibility_mode != 0
    }

    fn try_lex_operator(&mut self) -> PdfResult<ContentTokenOrUnknownOperator> {
        let start = self.cursor;

        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_alphanumeric() && b != b'*' {
                break;
            }

            self.next_byte();
        }

        let s = std::str::from_utf8(&self.buffer[start..self.cursor]).unwrap();

        if s == "true" {
            return Ok(ContentTokenOrUnknownOperator::Token(ContentToken::Object(
                Object::True,
            )));
        }

        if s == "false" {
            return Ok(ContentTokenOrUnknownOperator::Token(ContentToken::Object(
                Object::False,
            )));
        }

        if s == "null" {
            return Ok(ContentTokenOrUnknownOperator::Token(ContentToken::Object(
                Object::Null,
            )));
        }

        Ok(if let Ok(op) = Operator::from_str(s) {
            ContentTokenOrUnknownOperator::Token(ContentToken::Operator(op))
        } else {
            ContentTokenOrUnknownOperator::UnknownOperator(s.to_owned())
        })
    }

    fn next_token(&mut self) -> Option<PdfResult<ContentTokenOrUnknownOperator>> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b'"' | b'\'' | b'a'..=b'z' | b'A'..=b'Z') => Some(self.try_lex_operator()),
            Some(..) => Some(Ok(ContentTokenOrUnknownOperator::Token(
                ContentToken::Object(match self.lex_object() {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                }),
            ))),
            None => None,
        }
    }
}

impl LexBase for ContentLexer<'_> {
    fn buffer<'a>(&'a self) -> &'a [u8] {
        self.buffer.borrow()
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }
}

impl LexObject for ContentLexer<'_> {
    fn lex_dict(&mut self) -> PdfResult<Object> {
        Ok(Object::Dictionary(self.lex_dict_ignore_stream()?))
    }
}
