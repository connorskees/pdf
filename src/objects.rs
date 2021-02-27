use std::{collections::HashMap, convert::TryFrom, fmt};

use crate::{
    assert_reference, catalog::Rectangle, date::Date, stream::Stream, ParseError, PdfResult,
    Resolve,
};

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

impl Object {
    /// If self is an instance of `Object::Name`, returns whether or not
    /// the names are equivalent.
    ///
    /// Otherwise, returns false
    pub fn name_is(&self, name: &str) -> bool {
        if let Object::Name(name_two) = self {
            name == name_two
        } else {
            false
        }
    }
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

#[derive(Clone)]
pub struct Dictionary {
    dict: HashMap<String, Object>,
}

impl fmt::Debug for Dictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dictionary = f.debug_struct("Dictionary");

        for (key, value) in &self.dict {
            dictionary.field(key, &value);
        }

        dictionary.finish()
    }
}

impl Dictionary {
    pub fn new(dict: HashMap<String, Object>) -> Self {
        Self { dict }
    }

    pub fn entries(self) -> impl Iterator<Item = (String, Object)> {
        self.dict.into_iter()
    }

    pub fn get_stream(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<Stream>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_stream(obj))
            .transpose()
    }

    pub fn expect_stream(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Stream> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_stream(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_number(&mut self, key: &str, resolver: &mut dyn Resolve) -> PdfResult<Option<f32>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_number(obj))
            .transpose()
    }

    pub fn expect_number(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<f32> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_number(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    fn expect_name_is_value(
        &mut self,
        key: &'static str,
        val: &'static str,
        required: bool,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<()> {
        let type_val = self.get_name(key, resolver)?;

        match type_val {
            Some(name) if name != val => {
                return Err(ParseError::MismatchedTypeKey {
                    expected: val,
                    found: name,
                });
            }
            None if required => return Err(ParseError::MissingRequiredKey { key }),
            Some(..) | None => {}
        }

        Ok(())
    }

    pub fn expect_type(
        &mut self,
        ty: &'static str,
        resolver: &mut dyn Resolve,
        required: bool,
    ) -> PdfResult<()> {
        self.expect_name_is_value("Type", ty, required, resolver)
    }

    pub fn get_type_or_arr<T: fmt::Debug, S: Resolve + Sized>(
        &mut self,
        key: &'static str,
        resolver: &mut S,
        convert: impl Fn(&mut S, Object) -> PdfResult<T>,
    ) -> PdfResult<Option<TypeOrArray<T>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.get_type_or_arr(obj, convert))
            .transpose()
    }

    pub fn get_integer(&mut self, key: &str, resolver: &mut dyn Resolve) -> PdfResult<Option<i32>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_integer(obj))
            .transpose()
    }

    pub fn get_unsigned_integer(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<u32>> {
        self.dict
            .remove(key)
            .map(|obj| Ok(u32::try_from(resolver.assert_integer(obj)?)?))
            .transpose()
    }

    pub fn expect_integer(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<i32> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_integer(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn expect_unsigned_integer(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<u32> {
        self.dict
            .remove(key)
            .map(|obj| Ok(u32::try_from(resolver.assert_integer(obj)?)?))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_reference(&mut self, key: &str) -> PdfResult<Option<Reference>> {
        self.dict.remove(key).map(assert_reference).transpose()
    }

    pub fn get_string(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<String>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_string(obj))
            .transpose()
    }

    pub fn expect_string(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<String> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_string(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn expect_reference(&mut self, key: &'static str) -> PdfResult<Reference> {
        self.dict
            .remove(key)
            .map(assert_reference)
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_object(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<Object>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.resolve(obj))
            .transpose()
    }

    pub fn expect_object(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Object> {
        self.dict
            .remove(key)
            .map(|obj| resolver.resolve(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }

    pub fn get_name(&mut self, key: &str, resolver: &mut dyn Resolve) -> PdfResult<Option<String>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_name(obj))
            .transpose()
    }

    pub fn expect_name(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<String> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_name(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_dict(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<Dictionary>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_dict(obj))
            .transpose()
    }

    pub fn expect_dict(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Dictionary> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_dict(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_arr(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<Vec<Object>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_arr(obj))
            .transpose()
    }

    pub fn expect_arr(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Vec<Object>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_arr(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_bool(&mut self, key: &str, resolver: &mut dyn Resolve) -> PdfResult<Option<bool>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_bool(obj))
            .transpose()
    }
}

/// Non-native objects
impl Dictionary {
    pub fn get_rectangle(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Option<Rectangle>> {
        self.get_arr(key, resolver)?
            .map(|objs| Rectangle::from_arr(objs, resolver))
            .transpose()
    }

    pub fn expect_rectangle(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Rectangle> {
        Rectangle::from_arr(self.expect_arr(key, resolver)?, resolver)
    }

    pub fn get_date(&mut self, key: &str, resolver: &mut dyn Resolve) -> PdfResult<Option<Date>> {
        self.get_string(key, resolver)?
            .as_deref()
            .map(Date::from_str)
            .transpose()
    }

    pub fn expect_date(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Date> {
        Date::from_str(&self.expect_string(key, resolver)?)
    }
}
