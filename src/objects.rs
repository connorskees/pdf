use std::{borrow::Cow, collections::HashMap, convert::TryFrom, fmt, marker::PhantomData, rc::Rc};

use crate::{
    assert_reference, data_structures::Matrix, date::Date, stream::Stream, ParseError, PdfResult,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Object<'a> {
    Null,
    True,
    False,
    Integer(i32),
    Real(f32),
    String(String),
    Name(String),
    Array(Vec<Self>),
    Stream(Stream<'a>),
    Dictionary(Dictionary<'a>),
    Reference(Reference),
}

impl<'a> Object<'a> {
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum TypedReference<'a, T: FromObj<'a>> {
    Indirect {
        reference: Reference,
        _t: PhantomData<&'a T>,
    },
    Direct(T),
}

impl<'a, T: FromObj<'a> + Clone> TypedReference<'a, T> {
    // todo: cache somehow
    pub fn get_ref(&self, resolver: &mut dyn Resolve<'a>) -> PdfResult<Cow<T>> {
        match self {
            TypedReference::Indirect { reference, _t } => {
                let value = T::from_obj(Object::Reference(*reference), resolver)?;
                Ok(Cow::Owned(value))
            }
            TypedReference::Direct(t) => Ok(Cow::Borrowed(t)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Dictionary<'a> {
    dict: HashMap<String, Object<'a>>,
}

impl fmt::Debug for Dictionary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dict.is_empty() {
            write!(f, "Dictionary {{}}")?;
            return Ok(());
        }

        let mut dictionary = f.debug_struct("Dictionary");

        for (key, value) in &self.dict {
            dictionary.field(key, &value);
        }

        dictionary.finish()
    }
}

impl<'a> Dictionary<'a> {
    pub fn new(dict: HashMap<String, Object<'a>>) -> Self {
        Self { dict }
    }

    pub fn empty() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    pub fn entries(self) -> impl Iterator<Item = (String, Object<'a>)> {
        self.dict.into_iter()
    }

    pub fn get<T: FromObj<'a>>(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<T>> {
        self.dict
            .remove(key)
            .and_then(|obj| match resolver.resolve(obj) {
                Ok(obj) if obj == Object::Null => None,
                Ok(obj) => Some(T::from_obj(obj, resolver)),
                Err(e) => Some(Err(e)),
            })
            .transpose()
    }

    pub fn expect<T: FromObj<'a>>(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<T> {
        self.dict
            .remove(key)
            .map(|obj| T::from_obj(obj, resolver))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_stream(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<Stream<'a>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_stream(obj))
            .transpose()
    }

    pub fn expect_stream(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Stream<'a>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_stream(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_number(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<f32>> {
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
                anyhow::bail!(ParseError::MismatchedTypeKey {
                    expected: val,
                    found: name,
                });
            }
            None if required => anyhow::bail!(ParseError::MissingRequiredKey { key }),
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
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<Object<'a>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.resolve(obj))
            .transpose()
    }

    pub fn expect_object(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Object<'a>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.resolve(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }

    pub fn get_obj_cloned(&self, key: &str) -> Option<Object<'a>> {
        self.dict.get(key).cloned()
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
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<Dictionary<'a>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_dict_or_null(obj))
            .transpose()
            .map(Option::flatten)
    }

    pub fn expect_dict(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Dictionary<'a>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_dict(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_arr(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<Vec<Object<'a>>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_arr(obj))
            .transpose()
    }

    pub fn expect_arr(
        &mut self,
        key: &'static str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Vec<Object<'a>>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_arr(obj))
            .ok_or(ParseError::MissingRequiredKey { key })?
    }

    pub fn get_bool(
        &mut self,
        key: &str,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Option<bool>> {
        self.dict
            .remove(key)
            .map(|obj| resolver.assert_bool(obj))
            .transpose()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Name(pub String);

pub trait FromObj<'a>: Sized {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self>;
}

impl<'a> FromObj<'a> for i32 {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_integer(obj)
    }
}

impl<'a> FromObj<'a> for u32 {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_unsigned_integer(obj)
    }
}

impl<'a> FromObj<'a> for usize {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(u32::from_obj(obj, resolver)? as usize)
    }
}

impl<'a> FromObj<'a> for f32 {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_number(obj)
    }
}

impl<'a> FromObj<'a> for bool {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_bool(obj)
    }
}

impl<'a> FromObj<'a> for Name {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Name(resolver.assert_name(obj)?))
    }
}

impl<'a> FromObj<'a> for String {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_string(obj)
    }
}

impl<'a> FromObj<'a> for Stream<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_stream(obj)
    }
}

impl<'a> FromObj<'a> for Object<'a> {
    fn from_obj(obj: Object<'a>, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(obj)
    }
}

impl<'a> FromObj<'a> for Reference {
    fn from_obj(obj: Object<'a>, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        assert_reference(obj)
    }
}

impl<'a> FromObj<'a> for Dictionary<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        resolver.assert_dict(obj)
    }
}

impl<'a, T: FromObj<'a>> FromObj<'a> for Vec<T> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let arr = resolver.assert_arr(obj)?;
        arr.into_iter()
            .map(|obj| T::from_obj(obj, resolver))
            .collect()
    }
}

impl<'a, T: FromObj<'a>> FromObj<'a> for Rc<T> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Rc::new(T::from_obj(obj, resolver)?))
    }
}

impl<'a> FromObj<'a> for Date {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let s = resolver.assert_string(obj)?;
        Date::from_str(&s)
    }
}

impl<'a> FromObj<'a> for Matrix {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let arr = resolver.assert_arr(obj)?;
        Matrix::from_arr(arr, resolver)
    }
}

impl<'a, T: FromObj<'a>> FromObj<'a> for TypedReference<'a, T> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match obj {
            Object::Reference(reference) => Self::Indirect {
                reference,
                _t: PhantomData,
            },
            _ => Self::Direct(T::from_obj(obj, resolver)?),
        })
    }
}
