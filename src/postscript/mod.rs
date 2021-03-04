use std::borrow::Cow;

use crate::pdf_enum;

use error::{PostScriptError, PostScriptResult};

mod error;

#[derive(Debug)]
pub(crate) struct PostScriptLexer {
    buffer: Box<[u8]>,
    cursor: usize,
}

#[derive(Debug)]
pub(crate) enum PostScriptToken {
    Operator(PostScriptOperator),
    Real(f32),
    Integer(i32),
    OpenCurlyBrace,
    CloseCurlyBrace,
}

pdf_enum!(
    #[derive(Debug)]
    pub(crate) enum PostScriptOperator {
        // Arithmetic
        Abs = "abs",
        Add = "add",
        Atan = "atan",
        Ceiling = "ceiling",
        Cos = "cos",
        Cvi = "cvi",
        Cvr = "cvr",
        Div = "div",
        Exp = "exp",
        Floor = "floor",
        Idiv = "idiv",
        Ln = "ln",
        Log = "log",
        Mod = "mod",
        Mul = "mul",
        Neg = "neg",
        Round = "round",
        Sin = "sin",
        Sqrt = "sqrt",
        Sub = "sub",
        Truncate = "truncate",

        // Relational, boolean, and bitwise
        And = "and",
        Bitshift = "bitshift",
        Eq = "eq",
        False = "false",
        Ge = "ge",
        Gt = "gt",
        Le = "le",
        Lt = "lt",
        Ne = "ne",
        Not = "not",
        Or = "or",
        True = "true",
        Xor = "xor",

        // Conditional
        If = "if",
        Ifelse = "ifelse",

        // Stack
        Copy = "copy",
        Dup = "dup",
        Exch = "exch",
        Index = "index",
        Pop = "pop",
        Roll = "roll",
    }
);

// todo: case sensitive?
fn ident_token_from_bytes(bytes: &[u8]) -> PostScriptResult<PostScriptToken> {
    Ok(match bytes {
        b"abs" => PostScriptToken::Operator(PostScriptOperator::Abs),
        b"add" => PostScriptToken::Operator(PostScriptOperator::Add),
        b"atan" => PostScriptToken::Operator(PostScriptOperator::Atan),
        b"ceiling" => PostScriptToken::Operator(PostScriptOperator::Ceiling),
        b"cos" => PostScriptToken::Operator(PostScriptOperator::Cos),
        b"cvi" => PostScriptToken::Operator(PostScriptOperator::Cvi),
        b"cvr" => PostScriptToken::Operator(PostScriptOperator::Cvr),
        b"div" => PostScriptToken::Operator(PostScriptOperator::Div),
        b"exp" => PostScriptToken::Operator(PostScriptOperator::Exp),
        b"floor" => PostScriptToken::Operator(PostScriptOperator::Floor),
        b"idiv" => PostScriptToken::Operator(PostScriptOperator::Idiv),
        b"ln" => PostScriptToken::Operator(PostScriptOperator::Ln),
        b"log" => PostScriptToken::Operator(PostScriptOperator::Log),
        b"mod" => PostScriptToken::Operator(PostScriptOperator::Mod),
        b"mul" => PostScriptToken::Operator(PostScriptOperator::Mul),
        b"neg" => PostScriptToken::Operator(PostScriptOperator::Neg),
        b"round" => PostScriptToken::Operator(PostScriptOperator::Round),
        b"sin" => PostScriptToken::Operator(PostScriptOperator::Sin),
        b"sqrt" => PostScriptToken::Operator(PostScriptOperator::Sqrt),
        b"sub" => PostScriptToken::Operator(PostScriptOperator::Sub),
        b"truncate" => PostScriptToken::Operator(PostScriptOperator::Truncate),
        b"and" => PostScriptToken::Operator(PostScriptOperator::And),
        b"bitshift" => PostScriptToken::Operator(PostScriptOperator::Bitshift),
        b"eq" => PostScriptToken::Operator(PostScriptOperator::Eq),
        b"false" => PostScriptToken::Operator(PostScriptOperator::False),
        b"ge" => PostScriptToken::Operator(PostScriptOperator::Ge),
        b"gt" => PostScriptToken::Operator(PostScriptOperator::Gt),
        b"le" => PostScriptToken::Operator(PostScriptOperator::Le),
        b"lt" => PostScriptToken::Operator(PostScriptOperator::Lt),
        b"ne" => PostScriptToken::Operator(PostScriptOperator::Ne),
        b"not" => PostScriptToken::Operator(PostScriptOperator::Not),
        b"or" => PostScriptToken::Operator(PostScriptOperator::Or),
        b"true" => PostScriptToken::Operator(PostScriptOperator::True),
        b"xor" => PostScriptToken::Operator(PostScriptOperator::Xor),
        b"if" => PostScriptToken::Operator(PostScriptOperator::If),
        b"ifelse" => PostScriptToken::Operator(PostScriptOperator::Ifelse),
        b"copy" => PostScriptToken::Operator(PostScriptOperator::Copy),
        b"dup" => PostScriptToken::Operator(PostScriptOperator::Dup),
        b"exch" => PostScriptToken::Operator(PostScriptOperator::Exch),
        b"index" => PostScriptToken::Operator(PostScriptOperator::Index),
        b"pop" => PostScriptToken::Operator(PostScriptOperator::Pop),
        b"roll" => PostScriptToken::Operator(PostScriptOperator::Roll),
        _ => {
            return Err(PostScriptError::ParseError(Cow::Owned(format!(
                "Unrecognized operator: {:?}",
                bytes
            ))))
        }
    })
}

impl PostScriptLexer {
    pub fn new(buffer: Box<[u8]>) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek_byte() {
            if b.is_ascii_whitespace() {
                self.next_byte();
            }
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let b = self.buffer.get(self.cursor)?;

        self.cursor += 1;

        Some(*b)
    }

    fn peek_byte(&mut self) -> Option<u8> {
        self.buffer.get(self.cursor).cloned()
    }

    fn lex_ident(&mut self) -> PostScriptResult<PostScriptToken> {
        let start = self.cursor;

        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_alphabetic() {
                break;
            }

            self.next_byte();
        }

        ident_token_from_bytes(&self.buffer[start..=self.cursor])
    }

    fn lex_whole_number(&mut self) {
        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_digit() {
                break;
            }

            self.next_byte();
        }
    }

    fn consume_if_next_byte_is(&mut self, b: u8) -> bool {
        if self.peek_byte() == Some(b) {
            self.next_byte();
            return true;
        }

        false
    }

    fn lex_number(&mut self) -> PostScriptResult<PostScriptToken> {
        let start = self.cursor;
        self.consume_if_next_byte_is(b'-');
        self.lex_whole_number();

        if self.consume_if_next_byte_is(b'.') {
            self.lex_whole_number();

            return Ok(PostScriptToken::Real(fast_float::parse(
                &self.buffer[start..self.cursor],
            )?));
        }

        Ok(PostScriptToken::Integer(parse_integer(
            &self.buffer[start..self.cursor],
        )))
    }

    fn next_token(&mut self) -> Option<PostScriptResult<PostScriptToken>> {
        Some(match self.peek_byte()? {
            b'0'..=b'9' | b'-' | b'.' => self.lex_number(),
            b'a'..=b'z' | b'A'..=b'Z' => self.lex_ident(),
            b'{' => Ok(PostScriptToken::OpenCurlyBrace),
            b'}' => Ok(PostScriptToken::CloseCurlyBrace),
            b => todo!("unexpected token start {:?}", b),
        })
    }
}

// todo: overflow
fn parse_integer(bytes: &[u8]) -> i32 {
    let mut n = 0;

    for b in bytes {
        n *= 10;
        n += *b as i32;
    }

    n
}

impl Iterator for PostScriptLexer {
    type Item = PostScriptResult<PostScriptToken>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
