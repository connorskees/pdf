use std::borrow::Cow;

use crate::postscript::{PostScriptError, PostScriptResult};

#[derive(Debug, Clone)]
pub(crate) struct PostScriptFunctionLexer {
    buffer: Box<[u8]>,
    cursor: usize,
}

#[derive(Debug)]
pub(crate) enum PostScriptFunctionToken {
    Operator(PostScriptFunctionOperator),
    Real(f32),
    Integer(i32),
    OpenCurlyBrace,
    CloseCurlyBrace,
}

#[pdf_enum]
pub(crate) enum PostScriptFunctionOperator {
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

// todo: case sensitive?
fn ident_token_from_bytes(bytes: &[u8]) -> PostScriptResult<PostScriptFunctionToken> {
    Ok(match bytes {
        b"abs" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Abs),
        b"add" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Add),
        b"atan" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Atan),
        b"ceiling" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Ceiling),
        b"cos" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Cos),
        b"cvi" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Cvi),
        b"cvr" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Cvr),
        b"div" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Div),
        b"exp" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Exp),
        b"floor" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Floor),
        b"idiv" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Idiv),
        b"ln" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Ln),
        b"log" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Log),
        b"mod" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Mod),
        b"mul" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Mul),
        b"neg" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Neg),
        b"round" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Round),
        b"sin" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Sin),
        b"sqrt" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Sqrt),
        b"sub" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Sub),
        b"truncate" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Truncate),
        b"and" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::And),
        b"bitshift" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Bitshift),
        b"eq" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Eq),
        b"false" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::False),
        b"ge" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Ge),
        b"gt" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Gt),
        b"le" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Le),
        b"lt" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Lt),
        b"ne" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Ne),
        b"not" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Not),
        b"or" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Or),
        b"true" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::True),
        b"xor" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Xor),
        b"if" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::If),
        b"ifelse" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Ifelse),
        b"copy" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Copy),
        b"dup" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Dup),
        b"exch" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Exch),
        b"index" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Index),
        b"pop" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Pop),
        b"roll" => PostScriptFunctionToken::Operator(PostScriptFunctionOperator::Roll),
        _ => {
            return Err(PostScriptError::ParseError(Cow::Owned(format!(
                "Unrecognized operator: {:?}",
                bytes
            ))))
        }
    })
}

impl PostScriptFunctionLexer {
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

    fn lex_ident(&mut self) -> PostScriptResult<PostScriptFunctionToken> {
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

    fn lex_number(&mut self) -> PostScriptResult<PostScriptFunctionToken> {
        let start = self.cursor;
        self.consume_if_next_byte_is(b'-');
        self.lex_whole_number();

        if self.consume_if_next_byte_is(b'.') {
            self.lex_whole_number();

            return Ok(PostScriptFunctionToken::Real(fast_float::parse(
                &self.buffer[start..self.cursor],
            )?));
        }

        Ok(PostScriptFunctionToken::Integer(parse_integer(
            &self.buffer[start..self.cursor],
        )))
    }

    fn next_token(&mut self) -> Option<PostScriptResult<PostScriptFunctionToken>> {
        Some(match self.peek_byte()? {
            b'0'..=b'9' | b'-' | b'.' => self.lex_number(),
            b'a'..=b'z' | b'A'..=b'Z' => self.lex_ident(),
            b'{' => Ok(PostScriptFunctionToken::OpenCurlyBrace),
            b'}' => Ok(PostScriptFunctionToken::CloseCurlyBrace),
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

impl Iterator for PostScriptFunctionLexer {
    type Item = PostScriptResult<PostScriptFunctionToken>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
