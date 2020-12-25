use std::collections::HashMap;

use crate::{stream::Stream, Lexer, ParseError, PdfResult};

#[derive(Debug)]
pub enum ObjectType {
    Null,
    Boolean,
    Integer,
    Real,
    String,
    Name,
    Array,
    Stream,
    Dictionary,
    Reference,
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    True,
    False,
    Integer(i32),
    Real(f32),
    String(String),
    Name(String),
    Array(Vec<Self>),
    Stream(Stream),
    Dictionary(Dictionary),
    Reference(Reference),
}

/// A reference to a non-existing object is considered a `null`
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Reference {
    pub object_number: usize,
    pub generation: usize,
}

#[derive(Debug, Clone)]
pub enum TypeOrArray<T> {
    Type(T),
    Array(Vec<T>),
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    dict: HashMap<String, Object>,
}

impl Dictionary {
    pub fn new(dict: HashMap<String, Object>) -> Self {
        Self { dict }
    }

    pub fn get_stream(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<Stream>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_stream(obj))
            .transpose()
    }

    pub fn get_type_or_arr<T>(
        &mut self,
        key: &'static str,
        lexer: &mut Lexer,
        convert: impl Fn(&mut Lexer, Object) -> PdfResult<T>,
    ) -> PdfResult<Option<TypeOrArray<T>>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.get_type_or_arr(obj, convert))
            .transpose()
    }

    pub fn get_integer(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<i32>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_integer(obj))
            .transpose()
    }

    pub fn expect_integer(&mut self, key: &'static str, lexer: &mut Lexer) -> PdfResult<i32> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_integer(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_reference(&mut self, key: &str) -> PdfResult<Option<Reference>> {
        self.dict
            .remove(key)
            .map(Lexer::assert_reference)
            .transpose()
    }

    pub fn get_string(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<String>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_string(obj))
            .transpose()
    }

    pub fn expect_reference(&mut self, key: &'static str) -> PdfResult<Reference> {
        self.dict
            .remove(key)
            .map(Lexer::assert_reference)
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_object(&mut self, key: &str) -> Option<Object> {
        self.dict.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }

    pub fn get_name(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<String>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_name(obj))
            .transpose()
    }

    pub fn expect_name(&mut self, key: &'static str, lexer: &mut Lexer) -> PdfResult<String> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_name(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_dict(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<Dictionary>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_dict(obj))
            .transpose()
    }

    pub fn expect_dict(&mut self, key: &'static str, lexer: &mut Lexer) -> PdfResult<Dictionary> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_dict(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_arr(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<Vec<Object>>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_arr(obj))
            .transpose()
    }

    pub fn expect_arr(&mut self, key: &'static str, lexer: &mut Lexer) -> PdfResult<Vec<Object>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_arr(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_bool(&mut self, key: &str, lexer: &mut Lexer) -> PdfResult<Option<bool>> {
        self.dict
            .remove(key)
            .map(|obj| lexer.assert_bool(obj))
            .transpose()
    }
}
