#![feature(or_patterns)]
#![allow(dead_code)]
// TODO: consider verifying the file header

mod actions;
mod annotation;
mod catalog;
mod date;
mod error;
mod file_specification;
mod flate_decoder;
mod font;
mod function;
mod graphics_state_parameters;
mod halftones;
mod object_stream;
mod objects;
mod page;
mod stream;
mod structure;
mod trailer;
mod xref;

use std::{borrow::Cow, cell::RefCell, collections::HashMap, convert::TryFrom, io, rc::Rc};

use crate::{
    annotation::Annotation,
    catalog::{DocumentCatalog, GroupAttributes, InformationDictionary, Rectangle, Resources},
    error::{ParseError, PdfResult},
    object_stream::{ObjectStream, ObjectStreamDict, ObjectStreamParser},
    objects::{Dictionary, Object, ObjectType, Reference, TypeOrArray},
    page::{PageNode, PageObject, PageTree, PageTreeNode},
    stream::{decode_stream, Stream, StreamDict},
    trailer::Trailer,
    xref::{ByteOffset, TrailerOrOffset, Xref, XrefParser},
};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

pub(crate) const NUMBERS: &[u8] = b"0123456789";

#[track_caller]
pub(crate) fn assert_empty(dict: Dictionary) {
    if !dict.is_empty() {
        todo!("dict not empty: {:#?}", dict);
    }
}

#[macro_export]
macro_rules! pdf_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$doc:meta])*
                $variant:ident = $val:literal
            ),*,
            }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            $(
                $(#[$doc])*
                $variant
            ),*,
        }

        impl $name {
            pub fn from_str(s: &str) -> crate::PdfResult<Self> {
                Ok(match s {
                    $($val => Self::$variant),*,
                    _ => return Err(crate::ParseError::UnrecognizedVariant {
                        ty: stringify!($name),
                        found: s.to_owned(),
                    })
                })
            }
        }
    };
}

pub fn assert_reference(obj: Object) -> PdfResult<Reference> {
    match obj {
        Object::Reference(r) => Ok(r),
        found => Err(ParseError::MismatchedObjectType {
            expected: ObjectType::Reference,
            found,
        }),
    }
}

pub trait Resolve {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object>;

    fn assert_integer(&mut self, obj: Object) -> PdfResult<i32> {
        match obj {
            Object::Integer(i) => Ok(i),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_integer(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Integer,
                found,
            }),
        }
    }

    fn assert_unsigned_integer(&mut self, obj: Object) -> PdfResult<u32> {
        match obj {
            Object::Integer(i) => Ok(u32::try_from(i)?),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_unsigned_integer(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Integer,
                found,
            }),
        }
    }

    /// Either an integer, or a real
    fn assert_number(&mut self, obj: Object) -> PdfResult<f32> {
        match obj {
            Object::Integer(i) => Ok(i as f32),
            Object::Real(i) => Ok(i),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_number(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Real,
                found,
            }),
        }
    }

    fn assert_dict(&mut self, obj: Object) -> PdfResult<Dictionary> {
        match obj {
            Object::Dictionary(d) => Ok(d),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_dict(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Dictionary,
                found,
            }),
        }
    }

    fn assert_name(&mut self, obj: Object) -> PdfResult<String> {
        match obj {
            Object::Name(n) => Ok(n),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_name(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Name,
                found,
            }),
        }
    }

    fn assert_string(&mut self, obj: Object) -> PdfResult<String> {
        match obj {
            Object::String(s) => Ok(s),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_string(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::String,
                found,
            }),
        }
    }

    fn assert_arr(&mut self, obj: Object) -> PdfResult<Vec<Object>> {
        match obj {
            Object::Array(a) => Ok(a),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_arr(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Array,
                found,
            }),
        }
    }

    fn assert_bool(&mut self, obj: Object) -> PdfResult<bool> {
        match obj {
            Object::True => Ok(true),
            Object::False => Ok(false),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_bool(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Boolean,
                found,
            }),
        }
    }

    fn assert_stream(&mut self, obj: Object) -> PdfResult<Stream> {
        match obj {
            Object::Stream(s) => Ok(s),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_stream(obj)
            }
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Stream,
                found,
            }),
        }
    }

    /// Resolve all references
    fn resolve(&mut self, obj: Object) -> PdfResult<Object> {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.resolve(obj)
            }
            obj => Ok(obj),
        }
    }

    fn assert_or_null<T>(
        &mut self,
        obj: Object,
        convert: impl Fn(&mut Self, Object) -> PdfResult<T>,
    ) -> PdfResult<Option<T>>
    where
        Self: Sized,
    {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_or_null(obj, convert)
            }
            Object::Null => Ok(None),
            obj => Some(convert(self, obj)).transpose(),
        }
    }

    fn get_type_or_arr<T>(
        &mut self,
        obj: Object,
        convert: impl Fn(&mut Self, Object) -> PdfResult<T>,
    ) -> PdfResult<TypeOrArray<T>>
    where
        Self: Sized,
    {
        Ok(if let Object::Array(els) = obj {
            TypeOrArray::Array(
                els.into_iter()
                    .map(|el| convert(self, el))
                    .collect::<PdfResult<Vec<T>>>()?,
            )
        } else {
            TypeOrArray::Type(convert(self, obj)?)
        })
    }
}

trait Lex {
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
        self.buffer().get(self.cursor()).cloned().map(|b| {
            *self.cursor_mut() += 1;
            b
        })
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

    fn peek_byte(&self) -> Option<u8> {
        self.buffer().get(self.cursor()).cloned()
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

    fn expect_byte(&mut self, b: u8) -> PdfResult<()> {
        match self.next_byte() {
            Some(b2) if b == b2 => Ok(()),
            b2 => Err(ParseError::MismatchedByte {
                expected: b,
                found: b2,
            }),
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
                })
            }
        }

        self.skip_whitespace();

        Ok(())
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

    fn lex_object(&mut self) -> PdfResult<Object> {
        self.skip_whitespace();
        let obj = match self.peek_byte() {
            Some(b't') => self.lex_true(),
            Some(b'f') => self.lex_false(),
            Some(b'n') => self.lex_null(),
            Some(b'<') => self.lex_gt(),
            Some(b'+' | b'-' | b'0'..=b'9') => self.lex_number(),
            Some(b'(') => self.lex_string(),
            Some(b'/') => Ok(Object::Name(self.lex_name()?)),
            Some(b'[') => self.lex_array(),
            Some(b) => todo!("{:?}", b as char),
            None => todo!(),
        }?;
        self.skip_whitespace();
        Ok(obj)
    }

    /// Assumes leading 't' has not been consumed
    fn lex_true(&mut self) -> PdfResult<Object> {
        self.expect_bytes(b"true")?;

        Ok(Object::True)
    }

    /// Assumes leading 'f' has not been consumed
    fn lex_false(&mut self) -> PdfResult<Object> {
        self.expect_bytes(b"false")?;

        Ok(Object::False)
    }

    /// Assumes leading 'n' has not been consumed
    fn lex_null(&mut self) -> PdfResult<Object> {
        self.expect_bytes(b"null")?;

        Ok(Object::Null)
    }

    fn lex_gt(&mut self) -> PdfResult<Object> {
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

    fn lex_dict_ignore_stream(&mut self) -> PdfResult<Dictionary> {
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

    fn lex_dict(&mut self) -> PdfResult<Object>;

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

    fn lex_hex_string(&mut self) -> PdfResult<Object> {
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
            match self.next_byte() {
                Some(b'R') => {
                    return Ok(Object::Reference(Reference {
                        object_number: whole_number.parse::<usize>().unwrap(),
                        generation: generation.parse::<usize>().unwrap(),
                    }));
                }
                Some(..) | None => {
                    *self.cursor_mut() = whole_end_pos;
                }
            }
        }

        Ok(Object::Integer(
            whole_number.parse::<i32>().unwrap() * negative,
        ))
    }

    fn line_number(&self) -> usize {
        self.buffer().iter().filter(|&&c| c == b'\n').count()
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
                        Some(c) => todo!(
                            "unhandled escaped char {:?} on line {}",
                            c as char,
                            self.line_number()
                        ),
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

    fn lex_stream(&mut self, stream_dict: StreamDict) -> PdfResult<Stream> {
        self.expect_bytes(b"stream")?;
        self.expect_eol()?;

        let stream = self.get_byte_range(self.cursor(), self.cursor() + stream_dict.len);
        *self.cursor_mut() += stream_dict.len;

        self.expect_eol()?;

        self.expect_bytes(b"endstream")?;
        self.expect_eol()?;

        Ok(Stream {
            stream,
            dict: stream_dict,
        })
    }

    /// `start` is inclusive, `end` is exclusive
    /// 0 indexed
    // todo: DO NOT COPY
    fn get_byte_range(&self, start: usize, end: usize) -> Vec<u8> {
        if start == end {
            return Vec::new();
        }

        self.buffer()[start..end].to_vec()
    }

    fn peek_byte_offset(&self, offset: usize) -> Option<u8> {
        self.buffer().get(self.cursor() + offset).cloned()
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

impl Lex for Lexer {
    fn buffer(&self) -> &[u8] {
        &self.file
    }

    fn cursor(&self) -> usize {
        self.pos
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.pos
    }

    // TODO: move to Lex trait proper and restrain to where Self: Sized + Resolve
    fn lex_dict(&mut self) -> PdfResult<Object> {
        let dict = self.lex_dict_ignore_stream()?;

        if self.next_matches(b"stream") {
            let stream_dict = StreamDict::from_dict(dict, self)?;
            return Ok(Object::Stream(self.lex_stream(stream_dict)?));
        }

        Ok(Object::Dictionary(dict))
    }
}

pub struct Lexer {
    file: Vec<u8>,
    pos: usize,
    xref: Rc<Xref>,
    cached_object_streams: HashMap<usize, ObjectStreamParser>,
}

impl Lexer {
    pub fn new(file: Vec<u8>, xref: Rc<Xref>) -> io::Result<Self> {
        Ok(Self {
            file,
            xref,
            pos: 0,
            cached_object_streams: HashMap::new(),
        })
    }

    fn lex_object_stream(&mut self, byte_offset: usize) -> PdfResult<ObjectStream<'_>> {
        self.pos = byte_offset;
        self.read_obj_prelude()?;

        let object_stream_dict = ObjectStreamDict::from_dict(self.lex_dict_ignore_stream()?, self)?;
        let stream = self
            .lex_stream(object_stream_dict.stream_dict.clone())?
            .stream;

        self.read_obj_trailer()?;

        Ok(ObjectStream {
            stream: Cow::Owned(stream),
            dict: object_stream_dict,
        })
    }

    fn lex_trailer(&mut self, offset: usize) -> PdfResult<Trailer> {
        self.pos = offset;
        self.expect_bytes(b"trailer")?;
        self.skip_whitespace();

        let trailer_dict = self.lex_dict()?;
        let trailer = Trailer::from_dict(self.assert_dict(trailer_dict)?, self)?;

        Ok(trailer)
    }

    fn lex_object_from_object_stream(
        &mut self,
        byte_offset: usize,
        reference: Reference,
    ) -> PdfResult<Object> {
        let parser = match self.cached_object_streams.get_mut(&byte_offset) {
            Some(v) => v,
            None => {
                let ObjectStream { stream, dict } = self.lex_object_stream(byte_offset)?;

                let decoded_stream = decode_stream(
                    // SAFETY: the lexer does not mutate the underlying buffer
                    //
                    // We do this to avoid an unnecessary copy of the stream
                    unsafe { &*(&*stream as *const [u8]) },
                    &dict.stream_dict,
                    self,
                )?;

                let parser = ObjectStreamParser::new(decoded_stream, dict)?;

                self.cached_object_streams
                    .entry(byte_offset)
                    .or_insert(parser)
            }
        };

        parser.parse_object(reference)
    }

    fn lex_page_tree(&mut self, xref: &Xref, root_reference: Reference) -> PdfResult<PageNode> {
        if xref.get_offset(root_reference, self)?.is_none() {
            return Ok(PageNode::Root(Rc::new(RefCell::new(PageTree {
                kids: Vec::new(),
                pages: HashMap::new(),
                count: 0,
            }))));
        };

        let mut root_dict = self.assert_dict(Object::Reference(root_reference))?;
        let count = root_dict.expect_integer("Count", self)? as usize;
        let raw_kids = root_dict.expect_arr("Kids", self)?;

        root_dict.expect_type("Pages", self, true)?;

        assert_empty(root_dict);

        let root = PageNode::Root(Rc::new(RefCell::new(PageTree {
            count,
            pages: HashMap::new(),
            kids: Vec::new(),
        })));

        let mut pages = HashMap::new();

        pages.insert(root_reference, root.clone());

        let mut page_queue = raw_kids
            .into_iter()
            .map(assert_reference)
            .collect::<PdfResult<Vec<Reference>>>()?;

        while let Some(kid_ref) = page_queue.pop() {
            let mut kid_dict = self.assert_dict(Object::Reference(kid_ref))?;

            match kid_dict.expect_name("Type", self)?.as_ref() {
                "Pages" => {
                    self.lex_page_tree_node(kid_dict, kid_ref, &mut page_queue, &mut pages)?
                }
                "Page" => self.lex_page_object(kid_dict, kid_ref, &mut pages)?,
                found => {
                    return Err(ParseError::MismatchedTypeKey {
                        expected: "Page",
                        found: found.to_owned(),
                    })
                }
            };
        }

        match root.clone() {
            PageNode::Root(root) => {
                root.borrow_mut().pages = pages;
            }
            _ => unreachable!(),
        }

        Ok(root)
    }

    fn lex_page_object(
        &mut self,
        mut dict: Dictionary,
        kid_ref: Reference,
        pages: &mut HashMap<Reference, PageNode>,
    ) -> PdfResult<()> {
        let parent = dict.expect_reference("Parent")?;
        let last_modified = None;
        let resources = Resources::from_dict(dict.expect_dict("Resources", self)?, self)?;
        let media_box = Rectangle::from_arr(dict.expect_arr("MediaBox", self)?, self)?;
        let crop_box = dict
            .get_arr("CropBox", self)?
            .map(|objs| Rectangle::from_arr(objs, self))
            .transpose()?;
        let bleed_box = dict
            .get_arr("BleedBox", self)?
            .map(|objs| Rectangle::from_arr(objs, self))
            .transpose()?;
        let trim_box = dict
            .get_arr("TrimBox", self)?
            .map(|objs| Rectangle::from_arr(objs, self))
            .transpose()?;
        let art_box = dict
            .get_arr("ArtBox", self)?
            .map(|objs| Rectangle::from_arr(objs, self))
            .transpose()?;
        let box_color_info = None;
        let contents = dict.get_type_or_arr("Contents", self, Lexer::assert_stream)?;
        let rotate = dict.get_integer("Rotate", self)?.unwrap_or(0);
        let group = dict
            .get_dict("Group", self)?
            .map(|dict| GroupAttributes::from_dict(dict, self))
            .transpose()?;
        let thumb = None;
        let b = None;
        let dur = None;
        let trans = None;
        let annots = dict
            .get_arr("Annots", self)?
            .map(|annots| {
                annots
                    .into_iter()
                    .map(assert_reference)
                    .collect::<PdfResult<Vec<Reference>>>()
            })
            .transpose()?;
        let aa = None;
        let metadata = None;
        let piece_info = None;
        let struct_parents = dict.get_integer("StructParents", self)?;
        let id = None;
        let pz = None;
        let separation_info = None;
        let tabs = None;
        let template_instantiated = None;
        let pres_steps = None;
        let user_unit = None;
        let vp = None;

        assert_empty(dict);

        let parent = pages.get(&parent).unwrap().clone();

        let this_node = PageNode::Leaf(Rc::new(PageObject {
            parent: parent.clone(),
            last_modified,
            resources,
            media_box,
            crop_box,
            bleed_box,
            trim_box,
            art_box,
            box_color_info,
            contents,
            rotate,
            group,
            thumb,
            b,
            dur,
            trans,
            annots,
            aa,
            metadata,
            piece_info,
            struct_parents,
            id,
            pz,
            separation_info,
            tabs,
            template_instantiated,
            pres_steps,
            user_unit,
            vp,
        }));

        pages.insert(kid_ref, this_node.clone());

        match parent {
            PageNode::Node(node) => node.borrow_mut().kids.push(this_node),
            PageNode::Root(node) => node.borrow_mut().kids.push(this_node),
            PageNode::Leaf(..) => todo!("unreachable"),
        }

        Ok(())
    }

    fn lex_page_tree_node(
        &mut self,
        mut dict: Dictionary,
        kid_ref: Reference,
        page_queue: &mut Vec<Reference>,
        pages: &mut HashMap<Reference, PageNode>,
    ) -> PdfResult<()> {
        let kids = dict.expect_arr("Kids", self)?;
        let parent = dict.expect_reference("Parent")?;
        let count = dict.expect_integer("Count", self)? as usize;

        let parent = pages.get(&parent).unwrap().clone();

        let this_node = PageNode::Node(Rc::new(RefCell::new(PageTreeNode {
            kids: Vec::new(),
            parent: parent.clone(),
            count,
        })));

        match parent {
            PageNode::Node(node) => node.borrow_mut().kids.push(this_node.clone()),
            PageNode::Root(node) => node.borrow_mut().kids.push(this_node.clone()),
            PageNode::Leaf(..) => todo!("unreachable"),
        }

        pages.insert(kid_ref, this_node);

        page_queue.append(
            &mut kids
                .into_iter()
                .map(assert_reference)
                .collect::<PdfResult<Vec<Reference>>>()?,
        );

        Ok(())
    }
}

impl Resolve for Lexer {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object> {
        let init_pos = self.pos;

        self.pos = match Rc::clone(&self.xref).get_offset(reference, self)? {
            Some(ByteOffset::MainFile(p)) => p,
            Some(ByteOffset::ObjectStream { byte_offset, .. }) => {
                return self.lex_object_from_object_stream(byte_offset, reference);
            }
            None => return Ok(Object::Null),
        };

        self.read_obj_prelude()?;

        let obj = self.lex_object()?;

        self.read_obj_trailer()?;

        self.pos = init_pos;

        Ok(obj)
    }
}

pub struct Parser {
    lexer: Lexer,
    xref: Rc<Xref>,
    trailer: Trailer,
    catalog: DocumentCatalog,
    page_tree: PageNode,
}

impl Parser {
    pub fn new(p: &'static str) -> PdfResult<Self> {
        let file = std::fs::read(p)?;

        let xref_and_trailer = XrefParser::new(&file).read_xref()?;
        let xref = Rc::new(xref_and_trailer.xref);
        let mut lexer = Lexer::new(file, xref.clone())?;

        let trailer = match xref_and_trailer.trailer_or_offset {
            TrailerOrOffset::Offset(offset) => lexer.lex_trailer(offset)?,
            TrailerOrOffset::Trailer(trailer) => trailer,
        };

        let catalog = DocumentCatalog::from_dict(
            lexer.assert_dict(Object::Reference(trailer.root))?,
            &mut lexer,
        )?;

        let page_tree = lexer.lex_page_tree(&xref, catalog.pages)?;

        Ok(Self {
            lexer,
            xref,
            trailer,
            catalog,
            page_tree,
        })
    }

    pub fn info(&mut self) -> PdfResult<Option<InformationDictionary>> {
        Ok(Some(InformationDictionary::from_dict(
            self.lexer.assert_dict(match self.trailer.info {
                Some(r) => Object::Reference(r),
                None => return Ok(None),
            })?,
            &mut self.lexer,
        )?))
    }

    // todo: make this an iterator
    pub fn pages(&self) -> Vec<Rc<PageObject>> {
        self.page_tree.leaves()
    }

    pub fn page_annotations(&mut self, page: &PageObject) -> PdfResult<Option<Vec<Annotation>>> {
        if let Some(annots) = &page.annots {
            let annotations = annots
                .into_iter()
                .map(|&annot| {
                    let obj = self.lexer.lex_object_from_reference(annot)?;
                    let dict = self.lexer.assert_dict(obj)?;

                    Annotation::from_dict(dict, &mut self.lexer)
                })
                .collect::<PdfResult<Vec<Annotation>>>()?;

            return Ok(Some(annotations));
        }

        Ok(None)
    }

    pub fn run(mut self) -> PdfResult<Vec<Object>> {
        dbg!(self.info().unwrap());
        todo!()
    }
}

fn main() -> PdfResult<()> {
    let parser = Parser::new("test2.pdf")?;

    dbg!(parser.run().unwrap());

    Ok(())
}
