use std::convert::TryFrom;

use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType, Reference},
    stream::Stream,
};

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
}
