use std::{borrow::Cow, collections::HashMap};

use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, Reference},
    stream::{Stream, StreamDict},
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

pub(crate) trait LexBase<'a> {
    fn buffer(&self) -> &[u8];
    fn cursor(&self) -> usize;
    fn cursor_mut(&mut self) -> &mut usize;

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek_byte() {
            if Self::is_whitespace(b) {
                self.next_byte();
            } else if b == b'%' {
                self.next_byte();
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.buffer().get(self.cursor()).copied().map(|b| {
            *self.cursor_mut() += 1;
            b
        })
    }

    fn peek_byte(&self) -> Option<u8> {
        self.buffer().get(self.cursor()).copied()
    }

    fn peek_byte_offset(&self, offset: usize) -> Option<u8> {
        self.buffer().get(self.cursor() + offset).copied()
    }

    fn next_is_delimiter(&self) -> bool {
        self.peek_byte().map_or(false, Self::is_delimiter)
    }

    fn next_is_whitespace(&self) -> bool {
        self.peek_byte().map_or(false, Self::is_whitespace)
    }

    /// Whitespace chars are defined as
    ///
    /// * NUL             0x0
    /// * Horizontal tab  0x9
    /// * Line feed       0xa
    /// * Form feed       0xc
    /// * Carriage return 0xd
    /// * Space           0x20
    ///
    fn is_whitespace(b: u8) -> bool {
        matches!(b, b'\0' | 0x9 | b'\n' | FORM_FEED | b'\r' | b' ')
    }

    fn is_delimiter(b: u8) -> bool {
        matches!(
            b,
            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
        )
    }

    fn is_regular(b: u8) -> bool {
        !Self::is_whitespace(b) && !Self::is_delimiter(b)
    }

    /// `start` is inclusive, `end` is exclusive
    /// 0 indexed
    fn get_byte_range(&self, start: usize, end: usize) -> &'a [u8] {
        if start == end {
            return &[];
        }

        // SAFETY: this is only safe if we never modify the underlying buffer
        // TODO: remove, we can't enforce that invariant
        unsafe { &*(&self.buffer()[start..end] as *const _) }
    }

    /// Assumes the leading `%` has already been consumed
    fn skip_comment(&mut self) {
        while !self.next_is_eol() {
            self.next_byte();
        }
    }

    fn next_is_eol(&self) -> bool {
        match self.peek_byte() {
            Some(b'\r' | b'\n') => true,
            Some(..) => false,
            None => true,
        }
    }

    /// Does not modify the cursor
    fn next_matches(&mut self, bytes: &[u8]) -> bool {
        let start_pos = self.cursor();

        for &b in bytes {
            if Some(b) != self.next_byte() {
                *self.cursor_mut() = start_pos;
                return false;
            }
        }

        *self.cursor_mut() = start_pos;

        true
    }

    fn previous_byte(&mut self) -> Option<u8> {
        if self.cursor() == 0 {
            return None;
        }

        *self.cursor_mut() -= 1;

        self.buffer().get(self.cursor()).cloned()
    }

    fn next_byte_err(&mut self) -> PdfResult<u8> {
        self.buffer()
            .get(self.cursor())
            .cloned()
            .map(|b| {
                *self.cursor_mut() += 1;
                b
            })
            .ok_or(ParseError::UnexpectedEof)
    }

    fn expect_byte(&mut self, expected: u8) -> PdfResult<()> {
        match self.next_byte() {
            Some(found) if expected == found => Ok(()),
            found => Err(ParseError::MismatchedByte { expected, found }),
        }
    }

    fn expect_bytes(&mut self, bytes: &[u8]) -> PdfResult<()> {
        for &b in bytes {
            self.expect_byte(b)?;
        }

        Ok(())
    }

    fn expect_eol(&mut self) -> PdfResult<()> {
        match self.next_byte() {
            Some(b'\n') => {}
            Some(b'\r') => {
                if self.peek_byte() == Some(b'\n') {
                    self.next_byte();
                }
            }
            b => {
                return Err(ParseError::MismatchedByteMany {
                    expected: &[b'\n', b'\r'],
                    found: b,
                });
            }
        }

        Ok(())
    }

    fn line_number(&self) -> usize {
        self.buffer().iter().filter(|&&c| c == b'\n').count()
    }

    fn lex_name(&mut self) -> PdfResult<String> {
        self.expect_byte(b'/')?;

        let mut name = String::new();

        while let Some(b) = self.peek_byte() {
            if !Self::is_regular(b) {
                break;
            }

            self.next_byte();

            if b == b'#' {
                let mut val = match self.next_byte() {
                    Some(n @ b'0'..=b'9') => n - b'0',
                    Some(n @ b'a'..=b'f') => n - b'a' + 10,
                    Some(n @ b'A'..=b'F') => n - b'A' + 10,
                    Some(..) | None => todo!(),
                };

                let n = match self.next_byte() {
                    Some(n @ b'0'..=b'9') => n - b'0',
                    Some(n @ b'a'..=b'f') => n - b'a' + 10,
                    Some(n @ b'A'..=b'F') => n - b'A' + 10,
                    Some(..) | None => todo!(),
                };

                val *= 16;
                val += n;

                name.push(val as char);
            } else {
                name.push(b as char);
            }
        }

        Ok(name)
    }

    fn lex_string(&mut self) -> PdfResult<String> {
        self.expect_byte(b'(')?;

        let mut string = String::new();
        let mut num_open_parens = 0;

        while let Some(b) = self.peek_byte() {
            match b {
                b')' if num_open_parens == 0 => {
                    self.next_byte();
                    break;
                }
                b')' => {
                    num_open_parens -= 1;
                    string.push(')');
                }
                b'(' => {
                    num_open_parens += 1;
                    string.push('(');
                }
                b'\\' => {
                    self.next_byte();
                    match self.next_byte() {
                        Some(b'n') => string.push('\n'),
                        Some(b'r') => string.push('\r'),
                        Some(b't') => string.push('\t'),
                        Some(b'b') => string.push(BACKSPACE as char),
                        Some(b'f') => string.push(FORM_FEED as char),
                        Some(b'(') => string.push('('),
                        Some(b')') => string.push(')'),
                        Some(b'\\') => string.push('\\'),
                        Some(b'0') => string.push('\0'),
                        // TODO: do we skip whitespace after `\` in multiline string?
                        Some(b'\n' | b'\r') => self.skip_whitespace(),
                        // octal escape of the form `\ddd`
                        Some(c) => {
                            let mut n = c - b'0';

                            let digit_two =
                                self.next_byte().ok_or(ParseError::UnexpectedEof)? - b'0';
                            let digit_three =
                                self.next_byte().ok_or(ParseError::UnexpectedEof)? - b'0';

                            n *= 8;
                            n += digit_two;
                            n *= 8;
                            n += digit_three;

                            string.push(n as char);
                        }
                        None => todo!(),
                    }
                    continue;
                }
                _ => {
                    string.push(b as char);
                }
            }
            self.next_byte();
        }

        Ok(string)
    }

    // TODO: throw error on empty string
    fn lex_whole_number(&mut self) -> String {
        let mut whole_number = String::new();

        while let Some(b) = self.peek_byte() {
            if !b.is_ascii_digit() {
                break;
            }

            self.next_byte();

            whole_number.push(b as char);
        }

        whole_number
    }
}

pub(crate) trait LexObject<'a>: LexBase<'a> {
    fn lex_object(&mut self) -> PdfResult<Object<'a>> {
        self.skip_whitespace();
        let obj = match self.peek_byte() {
            Some(b't') => self.lex_true(),
            Some(b'f') => self.lex_false(),
            Some(b'n') => self.lex_null(),
            Some(b'<') => self.lex_gt(),
            Some(b'+' | b'-' | b'0'..=b'9' | b'.') => self.lex_number(),
            Some(b'(') => Ok(Object::String(self.lex_string()?)),
            Some(b'/') => Ok(Object::Name(self.lex_name()?)),
            Some(b'[') => self.lex_array(),
            Some(b) => todo!(
                "unexpected object start {:?} at line {}",
                b as char,
                self.line_number()
            ),
            None => todo!(),
        }?;
        self.skip_whitespace();
        Ok(obj)
    }

    /// Assumes leading 't' has not been consumed
    fn lex_true(&mut self) -> PdfResult<Object<'a>> {
        self.expect_bytes(b"true")?;

        Ok(Object::True)
    }

    /// Assumes leading 'f' has not been consumed
    fn lex_false(&mut self) -> PdfResult<Object<'a>> {
        self.expect_bytes(b"false")?;

        Ok(Object::False)
    }

    /// Assumes leading 'n' has not been consumed
    fn lex_null(&mut self) -> PdfResult<Object<'a>> {
        self.expect_bytes(b"null")?;

        Ok(Object::Null)
    }

    fn lex_gt(&mut self) -> PdfResult<Object<'a>> {
        match self.peek_byte_offset(1) {
            Some(b'<') => self.lex_dict(),
            Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') => self.lex_hex_string(),
            // special cased empty byte string, `<>`
            Some(b'>') => {
                self.next_byte();
                self.next_byte();
                Ok(Object::String(String::new()))
            }
            Some(b) => todo!("{}", b),
            None => todo!(),
        }
    }

    fn lex_dict_ignore_stream(&mut self) -> PdfResult<Dictionary<'a>> {
        self.expect_byte(b'<')?;
        self.expect_byte(b'<')?;
        self.skip_whitespace();

        let mut dict = HashMap::new();

        while let Some(b) = self.peek_byte() {
            if b == b'>' {
                self.next_byte();
                self.expect_byte(b'>')?;
                break;
            }

            let name = self.lex_name()?;
            let value = self.lex_object()?;
            self.skip_whitespace();
            dict.insert(name, value);
        }

        self.skip_whitespace();

        Ok(Dictionary::new(dict))
    }

    fn lex_dict(&mut self) -> PdfResult<Object<'a>>;

    // utf-16 <FEFF0043006F006C006C00610062006F007200610020004F0066006600690063006500200036002E0034>
    fn read_hex_char(&mut self, is_utf16: bool) -> char {
        let mut val: u32 = 0;
        let len = if is_utf16 { 4 } else { 2 };
        let mut counter = 0;

        while let Some(b) = self.peek_byte() {
            val *= 16;

            // if there is an odd number of bytes, we treat the last byte as if it were 0
            if b == b'>' {
                break;
            }

            self.next_byte();
            val += Self::hex_byte_to_digit(b) as u32;

            counter += 1;

            if counter == len {
                break;
            }
        }

        // todo: invalid chars will panic (i think this is only possible for utf16)
        std::char::from_u32(val).unwrap()
    }

    // todo: base 85?
    fn lex_hex_string(&mut self) -> PdfResult<Object<'a>> {
        self.expect_byte(b'<')?;

        let mut string = String::new();

        let is_utf16 = self.next_matches(b"feff") || self.next_matches(b"FEFF");

        if is_utf16 {
            *self.cursor_mut() += 4;
        }

        while let Some(b) = self.peek_byte() {
            if b == b'>' {
                self.next_byte();
                break;
            }

            string.push(self.read_hex_char(is_utf16));
        }

        Ok(Object::String(string))
    }

    // todo: scientific notation (1e2)
    // todo: radix numbers (16#FFFE)
    fn lex_number(&mut self) -> PdfResult<Object<'a>> {
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

        let whole_end_pos = self.cursor();

        if self.peek_byte() == Some(b'.') {
            self.next_byte();
            let decimal_number = format!("{}.{}", whole_number, self.lex_whole_number());
            return Ok(Object::Real(
                decimal_number.parse::<f32>().unwrap() * negative as f32,
            ));
        }

        self.skip_whitespace();
        if self
            .peek_byte()
            .map(char::from)
            .as_ref()
            .map(char::is_ascii_digit)
            .unwrap_or(false)
        {
            let generation = self.lex_whole_number();
            self.skip_whitespace();

            if self.next_byte() == Some(b'R')
                && (self.next_is_delimiter() || self.next_is_whitespace())
            {
                return Ok(Object::Reference(Reference {
                    object_number: whole_number.parse::<usize>()?,
                    generation: generation.parse::<usize>()?,
                }));
            }

            *self.cursor_mut() = whole_end_pos;
        }

        Ok(Object::Integer(whole_number.parse::<i32>()? * negative))
    }

    fn lex_array(&mut self) -> PdfResult<Object<'a>> {
        let mut arr = Vec::new();
        self.expect_byte(b'[')?;
        while let Some(b) = self.peek_byte() {
            if b == b']' {
                self.next_byte();
                break;
            }

            arr.push(self.lex_object()?);
        }

        Ok(Object::Array(arr))
    }

    fn lex_stream(&mut self, stream_dict: StreamDict<'a>) -> PdfResult<Stream<'a>> {
        self.expect_bytes(b"stream")?;
        self.expect_eol()?;

        let stream = self.get_byte_range(self.cursor(), self.cursor() + stream_dict.len);

        *self.cursor_mut() += stream_dict.len;

        self.skip_whitespace();

        self.expect_bytes(b"endstream")?;
        self.expect_eol()?;

        Ok(Stream {
            stream: Cow::Borrowed(stream),
            dict: stream_dict,
        })
    }

    fn hex_byte_to_digit(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => todo!(),
        }
    }

    fn read_obj_prelude(&mut self) -> PdfResult<()> {
        self.lex_whole_number();
        self.skip_whitespace();
        self.lex_whole_number();
        self.skip_whitespace();
        self.expect_bytes(b"obj")?;
        self.skip_whitespace();

        Ok(())
    }

    fn read_obj_trailer(&mut self) -> PdfResult<()> {
        self.skip_whitespace();
        self.expect_bytes(b"endobj")?;

        Ok(())
    }
}
