#![feature(or_patterns)]
// TODO: consider verifying the file header

mod catalog;
mod xref;

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

use crate::{
    catalog::Trailer,
    xref::{EntryKind, Xref, XrefEntry},
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

const START_XREF_SIGNATURE: &[u8; 9] = b"startxref";
const KILOBYTE: usize = 1024;

#[derive(Debug)]
enum ParseError {
    MismatchedByte {
        expected: u8,
        found: Option<u8>,
    },
    MismatchedByteMany {
        expected: &'static [u8],
        found: Option<u8>,
    },
    UnexpectedEof,
    IoError(io::Error),
    MismatchedObjectType {
        expected: ObjectType,
        found: Object,
    },
    MissingRequiredKey {
        key: &'static str,
    },
    Todo,
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

type PdfResult<T> = Result<T, ParseError>;

struct Name(String);

/// A reference to a non-existing object is considered a `null`
#[derive(Debug, Clone)]
struct Reference {
    object_number: usize,
    generation: usize,
}

#[derive(Debug)]
enum ObjectType {
    Null,
    Boolean,
    Integer,
    Real,
    Name,
    Array,
    Stream,
    Dictionary,
    Reference,
}

#[derive(Debug, Clone)]
enum Object {
    Null,
    True,
    False,
    Integer(i32),
    Real(f32),
    String(String),
    Name(String),
    Array(Vec<Self>),
    Stream(Vec<u8>),
    Dictionary(Dictionary),
    Reference(Reference),
}

#[derive(Debug, Clone)]
struct Dictionary {
    dict: HashMap<String, Object>,
}

impl Dictionary {
    pub fn new(dict: HashMap<String, Object>) -> Self {
        Self { dict }
    }

    fn swap<T>(t: Option<PdfResult<T>>) -> PdfResult<Option<T>> {
        match t {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn get_integer(&mut self, key: &str) -> PdfResult<Option<i32>> {
        Self::swap(self.dict.remove(key).map(Object::assert_integer))
    }

    pub fn expect_integer(&mut self, key: &'static str) -> PdfResult<i32> {
        self.dict
            .remove(key)
            .map(Object::assert_integer)
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_reference(&mut self, key: &str) -> PdfResult<Option<Reference>> {
        Self::swap(self.dict.remove(key).map(Object::assert_reference))
    }

    pub fn expect_reference(&mut self, key: &'static str) -> PdfResult<Reference> {
        self.dict
            .remove(key)
            .map(Object::assert_reference)
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_object(&mut self, key: &str) -> Option<Object> {
        self.dict.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }
}

impl Object {
    pub fn assert_integer(self) -> PdfResult<i32> {
        match self {
            Self::Integer(i) => Ok(i),
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Integer,
                found,
            }),
        }
    }

    pub fn assert_dict(self) -> PdfResult<Dictionary> {
        match self {
            Self::Dictionary(d) => Ok(d),
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Dictionary,
                found,
            }),
        }
    }

    pub fn assert_reference(self) -> PdfResult<Reference> {
        match self {
            Self::Reference(r) => Ok(r),
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Reference,
                found,
            }),
        }
    }
}

struct StreamDict {
    len: usize,
    filter: Option<Object>,
    decode_params: Option<Object>,
    f: Option<Object>,
    f_filter: Option<Object>,
    f_decode_params: Option<Object>,
    dl: Option<usize>,
}

impl StreamDict {
    pub fn from_dict(mut dict: Dictionary) -> PdfResult<Self> {
        let len = dict.get_integer("Length")?.ok_or(ParseError::Todo)? as usize;

        let filter = dict.get_object("Filter");
        let decode_params = dict.get_object("DecodeParms");
        let f = dict.get_object("F");
        let f_filter = dict.get_object("FFilter");
        let f_decode_params = dict.get_object("FDecodeParms");
        let dl = dict.get_integer("DL")?.map(|i| i as usize);

        if !dict.is_empty() {
            todo!()
        }

        Ok(Self {
            len,
            filter,
            decode_params,
            f,
            f_filter,
            f_decode_params,
            dl,
        })
    }
}

struct Lexer {
    file: Vec<u8>,
    pos: usize,
    objects: Vec<Object>,
}

impl Lexer {
    pub fn new(p: &'static str) -> io::Result<Self> {
        let mut file = Vec::new();
        File::open(p)?.read_to_end(&mut file)?;

        Ok(Self {
            file,
            pos: 0,
            objects: Vec::new(),
        })
    }

    fn lex_object(&mut self) -> PdfResult<Object> {
        self.skip_whitespace();
        match self.peek_byte() {
            Some(b't') => self.lex_true(),
            Some(b'f') => self.lex_false(),
            Some(b'n') => self.lex_null(),
            Some(b'<') => self.lex_gt(),
            Some(b'+' | b'-' | b'0'..=b'9') => self.lex_number(),
            Some(b'(') => self.lex_string(),
            Some(b'/') => Ok(Object::Name(self.lex_name()?)),
            Some(b'[') => self.lex_array(),
            Some(b's') => {
                let stream_dict = self.objects.last().unwrap().clone().assert_dict()?;
                self.lex_stream(StreamDict::from_dict(stream_dict)?)
            }
            Some(b) => todo!("{:?}", b as char),
            None => todo!(),
        }
    }

    /// Assumes leading 't' has not been consumed
    fn lex_true(&mut self) -> PdfResult<Object> {
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'u')?;
        self.expect_byte(b'e')?;

        Ok(Object::True)
    }

    /// Assumes leading 'f' has not been consumed
    fn lex_false(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'f')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b's')?;
        self.expect_byte(b'e')?;

        Ok(Object::False)
    }

    /// Assumes leading 'n' has not been consumed
    fn lex_null(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'n')?;
        self.expect_byte(b'u')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b'l')?;

        Ok(Object::Null)
    }

    fn lex_gt(&mut self) -> PdfResult<Object> {
        match self.peek_byte_offset(1) {
            Some(b'<') => self.lex_dict(),
            Some(b'0'..=b'9' | b'A'..=b'F') => self.lex_hex_string(),
            Some(..) | None => todo!(),
        }
    }

    fn lex_dict(&mut self) -> PdfResult<Object> {
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

        Ok(Object::Dictionary(Dictionary::new(dict)))
    }

    fn lex_hex_string(&mut self) -> PdfResult<Object> {
        self.expect_byte(b'<')?;

        let mut string = String::new();

        while let Some(b) = self.next_byte() {
            if b == b'>' {
                break;
            }

            if !b.is_ascii_hexdigit() {
                todo!()
            }

            let mut this_byte = Self::hex_byte_to_digit(b);

            this_byte *= 16;

            match self.next_byte() {
                Some(b @ (b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) => {
                    this_byte += Self::hex_byte_to_digit(b)
                }
                Some(..) | None => todo!(),
            }

            string.push(this_byte as char);
        }

        Ok(Object::String(string))
    }

    fn lex_number(&mut self) -> PdfResult<Object> {
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

        let whole_end_pos = self.pos;

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
            match self.next_byte() {
                Some(b'R') => {
                    dbg!(self.peek_byte());
                    return Ok(Object::Reference(Reference {
                        object_number: whole_number.parse::<usize>().unwrap(),
                        generation: generation.parse::<usize>().unwrap(),
                    }));
                }
                Some(..) | None => {
                    self.pos = whole_end_pos;
                }
            }
        }

        Ok(Object::Integer(
            whole_number.parse::<i32>().unwrap() * negative,
        ))
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

    fn lex_string(&mut self) -> PdfResult<Object> {
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
                        // TODO: do we skip whitespace after `\` in multiline string?
                        Some(b'\n' | b'\r') => self.skip_whitespace(),
                        // TODO: octal escape
                        Some(..) | None => todo!(),
                    }
                }
                _ => {
                    string.push(b as char);
                }
            }
        }

        Ok(Object::String(string))
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
                let mut val;

                match self.next_byte() {
                    Some(n @ b'0'..=b'9') => val = n - b'0',
                    Some(..) | None => todo!(),
                }

                match self.next_byte() {
                    Some(n @ b'0'..=b'9') => {
                        val *= 16;
                        val += n - b'0';
                    }
                    Some(..) | None => todo!(),
                }

                name.push(val as char);
            } else {
                name.push(b as char);
            }
        }

        Ok(name)
    }

    fn lex_array(&mut self) -> PdfResult<Object> {
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

    fn lex_stream(&mut self, stream_dict: StreamDict) -> PdfResult<Object> {
        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'm')?;
        self.expect_eol()?;

        let stream = self.get_byte_range(self.pos, self.pos + stream_dict.len);
        self.pos += stream_dict.len;

        self.expect_eol()?;

        self.expect_byte(b'e')?;
        self.expect_byte(b'n')?;
        self.expect_byte(b'd')?;
        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'm')?;
        self.expect_eol()?;

        Ok(Object::Stream(stream))
    }

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

    fn next_byte_err(&mut self) -> PdfResult<u8> {
        self.file
            .get(self.pos)
            .cloned()
            .map(|b| {
                self.pos += 1;
                b
            })
            .ok_or(ParseError::UnexpectedEof)
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.file.get(self.pos).cloned().map(|b| {
            self.pos += 1;
            b
        })
    }

    fn peek_byte(&self) -> Option<u8> {
        self.file.get(self.pos).cloned()
    }

    fn peek_byte_offset(&self, offset: usize) -> Option<u8> {
        self.file.get(self.pos + offset).cloned()
    }

    /// `start` is inclusive, `end` is exclusive
    /// 0 indexed
    fn get_byte_range(&self, start: usize, end: usize) -> Vec<u8> {
        if start == end {
            return Vec::new();
        }

        self.file[start..end].to_vec()
    }

    fn expect_byte(&mut self, b: u8) -> PdfResult<()> {
        match self.next_byte() {
            Some(b2) if b == b2 => Ok(()),
            b2 => Err(ParseError::MismatchedByte {
                expected: b,
                found: b2,
            }),
        }
    }

    fn expect_eol(&mut self) -> PdfResult<()> {
        match self.next_byte() {
            Some(b'\n') => Ok(()),
            Some(b'\r') => {
                if self.peek_byte() == Some(b'\n') {
                    self.next_byte();
                }
                Ok(())
            }
            b => Err(ParseError::MismatchedByteMany {
                expected: &[b'\n', b'\r'],
                found: b,
            }),
        }
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

    fn next_is_eol(&self) -> bool {
        match self.peek_byte() {
            Some(b'\r' | b'\n') => true,
            Some(..) => false,
            None => true,
        }
    }

    fn hex_byte_to_digit(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'z' => b - b'a',
            b'A'..=b'Z' => b - b'A',
            _ => todo!(),
        }
    }

    /// Assumes the leading `%` has already been consumed
    fn skip_comment(&mut self) {
        while !self.next_is_eol() {
            self.next_byte();
        }
    }

    /// We read backwards in 1024 byte chunks, looking for `"startxref"`
    fn read_xref(&mut self) -> PdfResult<Xref> {
        let mut pos = self.file.len() - 1;

        let idx = loop {
            if pos == 0 {
                todo!();
            }

            let next_pos = pos.saturating_sub(KILOBYTE - START_XREF_SIGNATURE.len());
            if let Some(start) = self.file[next_pos..=pos]
                .windows(START_XREF_SIGNATURE.len())
                .position(|window| window == START_XREF_SIGNATURE)
            {
                break start + next_pos;
            }

            pos = next_pos;
        };

        self.pos = idx;

        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'x')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'f')?;

        self.skip_whitespace();

        let xref_pos = self.lex_whole_number().parse::<usize>().unwrap();

        self.pos = xref_pos;

        self.lex_xref()
    }

    fn lex_trailer(&mut self) -> PdfResult<Trailer> {
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'i')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'r')?;
        self.skip_whitespace();

        let trailer_dict = self.lex_dict()?.assert_dict()?;
        let trailer = Trailer::from_dict(trailer_dict)?;

        Ok(trailer)
    }

    fn lex_xref(&mut self) -> PdfResult<Xref> {
        self.expect_byte(b'x')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'f')?;

        self.skip_whitespace();

        let mut objects = HashMap::new();

        loop {
            let idx_offset = self.lex_whole_number().parse::<usize>().unwrap();
            self.skip_whitespace();

            let num_of_entries = self.lex_whole_number().parse::<usize>().unwrap();
            self.expect_eol()?;

            objects.reserve(num_of_entries);

            for i in 0..num_of_entries {
                let byte_offset = self.lex_whole_number().parse::<usize>().unwrap();
                self.skip_whitespace();
                let generation = self.lex_whole_number().parse::<u16>().unwrap();
                self.skip_whitespace();
                let entry_kind = EntryKind::from_byte(self.next_byte_err()?)?;
                self.skip_whitespace();

                objects.insert(
                    i + idx_offset,
                    XrefEntry {
                        byte_offset,
                        generation,
                        kind: entry_kind,
                    },
                );
            }

            match self.peek_byte() {
                Some(b't') => break,
                Some(b'0'..=b'9') => continue,
                found => {
                    return Err(ParseError::MismatchedByteMany {
                        found,
                        expected: &[
                            b't', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
                        ],
                    })
                }
            }
        }

        Ok(Xref { objects })
    }
}

struct Parser {
    lexer: Lexer,
    xref: Xref,
}

impl Parser {
    pub fn new(p: &'static str) -> PdfResult<Self> {
        let mut lexer = Lexer::new(p)?;
        let xref = lexer.read_xref()?;
        let trailer = lexer.lex_trailer()?;

        dbg!(trailer);

        Ok(Self { lexer, xref })
    }

    pub fn run(mut self) -> PdfResult<Vec<Object>> {
        todo!()
    }
}

fn main() -> PdfResult<()> {
    let mut parser = Parser::new("hello-world.pdf")?;

    dbg!(parser.run().unwrap());

    Ok(())
}
