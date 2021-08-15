use std::borrow::Cow;

use crate::{error::PdfResult, lex::LexBase};

use super::{
    object::{Container, StringIndex},
    PostScriptObject, PostScriptString, PostscriptOperator,
};

fn ident_token_from_bytes(bytes: &[u8]) -> PdfResult<PostScriptObject> {
    Ok(PostScriptObject::Operator(match bytes {
        b"abs" => PostscriptOperator::Abs,
        b"add" => PostscriptOperator::Add,
        b"dict" => PostscriptOperator::Dict,
        b"begin" => PostscriptOperator::Begin,
        b"dup" => PostscriptOperator::Dup,
        b"def" => PostscriptOperator::Def,
        b"readonly" => PostscriptOperator::ReadOnly,
        b"executeonly" => PostscriptOperator::ExecuteOnly,
        b"noaccess" => PostscriptOperator::NoAccess,
        b"false" => PostscriptOperator::False,
        b"true" => PostscriptOperator::True,
        b"end" => PostscriptOperator::End,
        b"currentfile" => PostscriptOperator::CurrentFile,
        b"eexec" => PostscriptOperator::EExec,
        b"currentdict" => PostscriptOperator::CurrentDict,
        b"string" => PostscriptOperator::String,
        b"exch" => PostscriptOperator::Exch,
        b"readstring" => PostscriptOperator::ReadString,
        b"pop" => PostscriptOperator::Pop,
        b"put" => PostscriptOperator::Put,
        b"known" => PostscriptOperator::Known,
        b"not" => PostscriptOperator::Not,
        b"get" => PostscriptOperator::Get,
        b"exec" => PostscriptOperator::Exec,
        b"ifelse" => PostscriptOperator::IfElse,
        b"lt" => PostscriptOperator::Lt,
        b"array" => PostscriptOperator::Array,
        b"index" => PostscriptOperator::Index,
        b"definefont" => PostscriptOperator::DefineFont,
        b"mark" => PostscriptOperator::Mark,
        b"closefile" => PostscriptOperator::CloseFile,
        b"findresource" => PostscriptOperator::FindResource,
        literal => {
            // todo: only to detect unimplemented operators
            match literal {
                b"StandardEncoding" | b"|" | b"|-" | b"-|" | b"systemdict" => {}
                found => todo!("{:?}", String::from_utf8_lossy(found)),
            }

            return Ok(PostScriptObject::Literal(PostScriptString::from_bytes(
                literal.to_vec(),
            )));
        }
    }))
}

#[derive(Debug)]
pub(super) struct PostScriptLexer<'a> {
    cursor: usize,
    buffer: Cow<'a, [u8]>,

    /// Not excessively pretty, but we intern strings inside the lexer
    pub(super) strings: Container<StringIndex, PostScriptString>,
}

impl<'a> PostScriptLexer<'a> {
    pub fn new(buffer: Cow<'a, [u8]>) -> Self {
        Self {
            buffer,
            cursor: 0,
            strings: Container::new(),
        }
    }

    pub fn reset_buffer(&mut self, buffer: Cow<'a, [u8]>) {
        self.cursor = 0;
        self.buffer = buffer;
    }

    pub fn buffer_from_cursor(&mut self) -> &[u8] {
        self.skip_whitespace();
        &self.buffer[self.cursor..]
    }

    pub fn get_range_from_cursor(&mut self, length: usize) -> (&[u8], bool) {
        if self.cursor + length >= self.buffer.len() {
            (&self.buffer[self.cursor..], false)
        } else {
            (&self.buffer[self.cursor..(self.cursor + length)], true)
        }
    }

    fn lex_object(&mut self) -> PdfResult<Option<PostScriptObject>> {
        self.skip_whitespace();

        Ok(Some(match self.peek_byte() {
            Some(b'0'..=b'9' | b'.' | b'+') => self.lex_number()?,
            Some(b'/') => {
                let name = self.lex_name()?.into_bytes();
                PostScriptObject::Name(PostScriptString::from_bytes(name))
            }
            Some(b'(') => {
                let s = self.lex_string()?.into_bytes();
                PostScriptObject::String(self.strings.insert(PostScriptString::from_bytes(s)))
            }
            Some(b'<') => self.lex_gt()?,
            Some(b'-') => match self.peek_byte_offset(1) {
                Some(b'0'..=b'9') => self.lex_number()?,
                _ => self.lex_operator()?,
            },
            Some(..) => self.lex_operator()?,
            None => return Ok(None),
        }))
    }

    fn lex_gt(&mut self) -> PdfResult<PostScriptObject> {
        todo!()
    }

    fn lex_operator(&mut self) -> PdfResult<PostScriptObject> {
        match self.peek_byte() {
            Some(b'[') => {
                self.next_byte();
                return Ok(PostScriptObject::Operator(PostscriptOperator::ArrayStart));
            }
            Some(b']') => {
                self.next_byte();
                return Ok(PostScriptObject::Operator(PostscriptOperator::ArrayEnd));
            }
            Some(b'{') => {
                self.next_byte();
                return Ok(PostScriptObject::Operator(
                    PostscriptOperator::ProcedureStart,
                ));
            }
            Some(b'}') => {
                self.next_byte();
                return Ok(PostScriptObject::Operator(PostscriptOperator::ProcedureEnd));
            }
            Some(b'<') => todo!(),
            Some(..) => {}
            None => todo!(),
        }

        let start = self.cursor;

        while let Some(b) = self.peek_byte() {
            if !Self::is_regular(b) {
                break;
            }

            self.next_byte();
        }

        ident_token_from_bytes(&self.buffer[start..self.cursor])
    }

    fn lex_number(&mut self) -> PdfResult<PostScriptObject> {
        let negative = match self.peek_byte() {
            Some(b'+') => {
                self.next_byte();
                1
            }
            Some(b'-') => {
                self.next_byte();
                -1
            }
            Some(..) => 1,
            None => unreachable!(),
        };

        let whole_number = self.lex_whole_number();

        if self.peek_byte() == Some(b'.') {
            self.next_byte();
            let decimal_number = format!("{}.{}", whole_number, self.lex_whole_number());
            return Ok(PostScriptObject::Float(
                decimal_number.parse::<f32>().unwrap() * negative as f32,
            ));
        }

        Ok(PostScriptObject::Int(
            whole_number.parse::<i32>()? * negative,
        ))
    }

    fn lex_null(&mut self) -> PdfResult<PostScriptObject> {
        todo!()
    }

    fn lex_true(&mut self) -> PdfResult<PostScriptObject> {
        todo!()
    }

    fn lex_false(&mut self) -> PdfResult<PostScriptObject> {
        todo!()
    }

    fn lex_array(&mut self) -> PdfResult<PostScriptObject> {
        todo!()
    }
}

impl<'a> LexBase<'a> for PostScriptLexer<'_> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }
}

impl<'a> Iterator for PostScriptLexer<'a> {
    type Item = PdfResult<PostScriptObject>;
    fn next(&mut self) -> Option<Self::Item> {
        self.lex_object().transpose()
    }
}
