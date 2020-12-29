#![feature(or_patterns)]
#![allow(dead_code)]
// TODO: consider verifying the file header

mod catalog;
mod date;
mod error;
mod file_specification;
mod font;
mod function;
mod graphics_state_parameters;
mod halftones;
mod objects;
mod page;
mod stream;
mod xref;

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{self, Read},
    rc::Rc,
};

use {
    catalog::{DocumentCatalog, InformationDictionary, Rectangle, Resources},
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType, Reference, TypeOrArray},
    page::{PageNode, PageObject, PageTree, PageTreeNode},
    stream::{Stream, StreamDict},
    xref::XrefParser,
};

use catalog::GroupAttributes;

use crate::{catalog::Trailer, xref::Xref};

const FORM_FEED: u8 = b'\x0C';
const BACKSPACE: u8 = b'\x08';

pub(crate) const NUMBERS: &[u8] = b"0123456789";

#[track_caller]
fn assert_empty(dict: Dictionary) {
    if !dict.is_empty() {
        todo!("dict not empty: {:#?}", dict);
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
}

pub struct Lexer {
    file: Vec<u8>,
    pos: usize,
    xref: Rc<Xref>,
}

impl Lexer {
    pub fn new(file: Vec<u8>, xref: Rc<Xref>) -> io::Result<Self> {
        Ok(Self { file, xref, pos: 0 })
    }

    fn next_matches(&mut self, bytes: &[u8]) -> bool {
        let start_pos = self.pos;

        for &b in bytes {
            if Some(b) != self.next_byte() {
                self.pos = start_pos;
                return false;
            }
        }

        self.pos = start_pos;

        true
    }

    fn get_type_or_arr<T>(
        &mut self,
        obj: Object,
        convert: impl Fn(&mut Lexer, Object) -> PdfResult<T>,
    ) -> PdfResult<TypeOrArray<T>> {
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

    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object> {
        self.pos = match self.xref.get_offset(reference) {
            Some(p) => p,
            None => return Ok(Object::Null),
        };

        self.lex_whole_number();
        self.skip_whitespace();
        self.lex_whole_number();
        self.skip_whitespace();
        self.expect_byte(b'o')?;
        self.expect_byte(b'b')?;
        self.expect_byte(b'j')?;
        self.skip_whitespace();

        let obj = self.lex_object()?;

        self.skip_whitespace();
        self.expect_byte(b'e')?;
        self.expect_byte(b'n')?;
        self.expect_byte(b'd')?;
        self.expect_byte(b'o')?;
        self.expect_byte(b'b')?;
        self.expect_byte(b'j')?;

        Ok(obj)
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

        self.skip_whitespace();

        if self.next_matches(b"stream") {
            let stream_dict = StreamDict::from_dict(Dictionary::new(dict), self)?;
            return self.lex_stream(stream_dict);
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
                    self.next_byte();
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

        Ok(Object::Stream(Stream {
            stream,
            dict: stream_dict,
        }))
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

    fn hex_byte_to_digit(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'z' => b - b'a',
            b'A'..=b'Z' => b - b'A',
            _ => todo!(),
        }
    }

    fn lex_trailer(&mut self, offset: usize) -> PdfResult<Trailer> {
        self.pos = offset;
        self.expect_byte(b't')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'i')?;
        self.expect_byte(b'l')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'r')?;
        self.skip_whitespace();

        let trailer_dict = self.lex_dict()?;
        let trailer = Trailer::from_dict(self.assert_dict(trailer_dict)?, self)?;

        Ok(trailer)
    }

    fn lex_page_tree(&mut self, xref: &Xref, root_reference: Reference) -> PdfResult<PageNode> {
        if xref.get_offset(root_reference).is_none() {
            return Ok(PageNode::Root(Rc::new(RefCell::new(PageTree {
                kids: Vec::new(),
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
            kids: Vec::new(),
        })));

        // let mut page_refs = HashMap::new();
        let mut pages = HashMap::new();

        pages.insert(root_reference, root.clone());

        let mut page_queue = raw_kids
            .into_iter()
            .map(Self::assert_reference)
            .collect::<PdfResult<Vec<Reference>>>()?;

        while let Some(kid_ref) = page_queue.pop() {
            let mut kid_dict = self.assert_dict(Object::Reference(kid_ref))?;

            match kid_dict.expect_name("Type", self)?.as_ref() {
                "Pages" => {
                    self.lex_page_tree_node(kid_dict, kid_ref, &mut page_queue, &mut pages)?
                }
                "Page" => self.lex_page_object(kid_dict, &mut pages)?,
                found => {
                    return Err(ParseError::MismatchedTypeKey {
                        expected: "Page",
                        found: found.to_owned(),
                    })
                }
            };
        }

        Ok(root)
    }

    fn lex_page_object(
        &mut self,
        mut dict: Dictionary,
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
        let annots = None;
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
                .map(Self::assert_reference)
                .collect::<PdfResult<Vec<Reference>>>()?,
        );

        Ok(())
    }
}

impl Lexer {
    pub fn assert_integer(&mut self, obj: Object) -> PdfResult<i32> {
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

    /// Either an integer, or a real
    pub fn assert_number(&mut self, obj: Object) -> PdfResult<f32> {
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

    pub fn assert_dict(&mut self, obj: Object) -> PdfResult<Dictionary> {
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

    pub fn assert_reference(obj: Object) -> PdfResult<Reference> {
        match obj {
            Object::Reference(r) => Ok(r),
            found => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Reference,
                found,
            }),
        }
    }

    pub fn assert_name(&mut self, obj: Object) -> PdfResult<String> {
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

    pub fn assert_string(&mut self, obj: Object) -> PdfResult<String> {
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

    pub fn assert_arr(&mut self, obj: Object) -> PdfResult<Vec<Object>> {
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

    pub fn assert_bool(&mut self, obj: Object) -> PdfResult<bool> {
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

    pub fn assert_stream(&mut self, obj: Object) -> PdfResult<Stream> {
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
    pub fn resolve(&mut self, obj: Object) -> PdfResult<Object> {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.resolve(obj)
            }
            obj => Ok(obj),
        }
    }
}

struct Parser {
    lexer: Lexer,
    xref: Rc<Xref>,
    trailer: Trailer,
    catalog: DocumentCatalog,
    page_tree: PageNode,
}

impl Parser {
    pub fn new(p: &'static str) -> PdfResult<Self> {
        let mut file = Vec::new();
        File::open(p)?.read_to_end(&mut file)?;

        let (xref, trailer_offset) = XrefParser::new(&file).read_xref()?;
        let xref = Rc::new(xref);
        let mut lexer = Lexer::new(file, xref.clone())?;
        let trailer = lexer.lex_trailer(trailer_offset)?;
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

    pub fn run(self) -> PdfResult<Vec<Object>> {
        dbg!(self.page_tree);
        todo!()
    }
}

fn main() -> PdfResult<()> {
    let parser = Parser::new("hello-world.pdf")?;

    dbg!(parser.run().unwrap());

    Ok(())
}
