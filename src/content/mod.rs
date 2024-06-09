use std::borrow::{Borrow, Cow};

use crate::{
    error::PdfResult,
    lex::{LexBase, LexObject},
    objects::Object,
};

pub(crate) use operator::PdfGraphicsOperator;
pub(crate) use stream::ContentStream;

mod operator;
mod stream;

pub struct ContentLexer<'a> {
    pub(crate) buffer: Cow<'a, [u8]>,
    cursor: usize,

    /// If >0, unrecognized operators will be ignored
    ///
    /// Set to true when encountering a `BX` operator, and to false
    /// when an `EX` operator is encountered
    in_compatibility_mode: u128,
}

#[derive(Debug, PartialEq)]
pub enum ContentToken<'a> {
    Object(Object<'a>),
    Operator(PdfGraphicsOperator),
}

#[derive(Debug)]
enum ContentTokenOrUnknownOperator<'a> {
    Token(ContentToken<'a>),
    UnknownOperator(String),
}

impl<'a> Iterator for ContentLexer<'a> {
    type Item = PdfResult<ContentToken<'a>>;

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

    pub fn debug_contents(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.buffer)
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

    fn try_lex_operator(&mut self) -> PdfResult<ContentTokenOrUnknownOperator<'a>> {
        let start = self.cursor;

        while let Some(b) = self.peek_byte() {
            // terminal characters that end operators but are not alphanumeric
            if b == b'*' || b == b'\'' || b == b'"' {
                self.next_byte();
                break;
            }

            if !b.is_ascii_alphanumeric() {
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

        Ok(if let Ok(op) = PdfGraphicsOperator::from_str(s) {
            ContentTokenOrUnknownOperator::Token(ContentToken::Operator(op))
        } else {
            ContentTokenOrUnknownOperator::UnknownOperator(s.to_owned())
        })
    }

    fn next_token(&mut self) -> Option<PdfResult<ContentTokenOrUnknownOperator<'a>>> {
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

impl<'a> LexBase<'a> for ContentLexer<'_> {
    fn buffer(&self) -> &[u8] {
        self.buffer.borrow()
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }
}

impl<'a> LexObject<'a> for ContentLexer<'_> {
    fn lex_dict(&mut self) -> PdfResult<Object<'a>> {
        Ok(Object::Dictionary(self.lex_dict_ignore_stream()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rg_operator_is_not_parsed_as_reference() {
        let buffer = b"1 1 1 RG";

        let tokens = ContentLexer::new(Cow::Borrowed(buffer))
            .collect::<PdfResult<Vec<ContentToken>>>()
            .unwrap();

        assert_eq!(
            tokens,
            vec![
                ContentToken::Object(Object::Integer(1)),
                ContentToken::Object(Object::Integer(1)),
                ContentToken::Object(Object::Integer(1)),
                ContentToken::Operator(PdfGraphicsOperator::RG)
            ]
        );
    }

    #[test]
    fn empty_line() {
        let buffer = b"\n\n  \n\n";

        let tokens = ContentLexer::new(Cow::Borrowed(buffer))
            .collect::<PdfResult<Vec<ContentToken>>>()
            .unwrap();

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn quote_operators() {
        let buffer = b"( )'\"";

        let tokens = ContentLexer::new(Cow::Borrowed(buffer))
            .collect::<PdfResult<Vec<ContentToken>>>()
            .unwrap();

        assert_eq!(
            tokens,
            vec![
                ContentToken::Object(Object::String(" ".to_owned())),
                ContentToken::Operator(PdfGraphicsOperator::single_quote),
                ContentToken::Operator(PdfGraphicsOperator::double_quote),
            ]
        );
    }

    #[test]
    fn no_space_after_star_operator() {
        let buffer = b"b*RG";

        let tokens = ContentLexer::new(Cow::Borrowed(buffer))
            .collect::<PdfResult<Vec<ContentToken>>>()
            .unwrap();

        assert_eq!(
            tokens,
            vec![
                ContentToken::Operator(PdfGraphicsOperator::b_star),
                ContentToken::Operator(PdfGraphicsOperator::RG),
            ]
        );
    }
}
