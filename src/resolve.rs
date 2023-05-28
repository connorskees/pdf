use std::convert::TryFrom;

use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType, Reference},
    stream::Stream,
};

pub trait Resolve<'a> {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object<'a>>;

    fn assert_integer(&mut self, obj: Object) -> PdfResult<i32> {
        match obj {
            Object::Integer(i) => Ok(i),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_integer(obj)
            }
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Integer,
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
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Integer,
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
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Real,
            }),
        }
    }

    fn assert_dict(&mut self, obj: Object<'a>) -> PdfResult<Dictionary<'a>> {
        match obj {
            Object::Dictionary(d) => Ok(d),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_dict(obj)
            }
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Dictionary,
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
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Name,
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
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::String,
            }),
        }
    }

    fn assert_arr(&mut self, obj: Object<'a>) -> PdfResult<Vec<Object<'a>>> {
        match obj {
            Object::Array(a) => Ok(a),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_arr(obj)
            }
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Array,
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
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Boolean,
            }),
        }
    }

    fn assert_stream(&mut self, obj: Object<'a>) -> PdfResult<Stream<'a>> {
        match obj {
            Object::Stream(s) => Ok(s),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_stream(obj)
            }
            _ => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Stream,
            }),
        }
    }

    /// Resolve all references
    fn resolve(&mut self, obj: Object<'a>) -> PdfResult<Object<'a>> {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.resolve(obj)
            }
            obj => Ok(obj),
        }
    }

    fn assert_dict_or_null(&mut self, obj: Object<'a>) -> PdfResult<Option<Dictionary<'a>>> {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_dict_or_null(obj)
            }
            Object::Null => Ok(None),
            obj => Some(self.assert_dict(obj)).transpose(),
        }
    }

    fn assert_number_or_null(&mut self, obj: Object) -> PdfResult<Option<f32>> {
        match obj {
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_number_or_null(obj)
            }
            Object::Null => Ok(None),
            obj => Some(self.assert_number(obj)).transpose(),
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
