use std::convert::TryFrom;

use crate::{
    error::PdfResult,
    objects::{Dictionary, Object, Reference},
    stream::Stream,
};

pub trait Resolve<'a> {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object<'a>>;

    /// Whether or not the reference points to an existing object
    fn reference_exists(&mut self, reference: Reference) -> PdfResult<bool>;

    fn assert_integer(&mut self, obj: Object) -> PdfResult<i32> {
        match obj {
            Object::Integer(i) => Ok(i),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_integer(obj)
            }
            obj => anyhow::bail!("expected integer, found {:?}", obj),
        }
    }

    fn assert_unsigned_integer(&mut self, obj: Object) -> PdfResult<u32> {
        match obj {
            Object::Integer(i) => Ok(u32::try_from(i)?),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_unsigned_integer(obj)
            }
            obj => anyhow::bail!("expected unsigned integer, found {:?}", obj),
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
            obj => anyhow::bail!("expected real, found {:?}", obj),
        }
    }

    fn assert_dict(&mut self, obj: Object<'a>) -> PdfResult<Dictionary<'a>> {
        match obj {
            Object::Dictionary(d) => Ok(d),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_dict(obj)
            }
            obj => anyhow::bail!("expected dictionary, found {:?}", obj),
        }
    }

    fn assert_name(&mut self, obj: Object) -> PdfResult<String> {
        match obj {
            Object::Name(n) => Ok(n),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_name(obj)
            }
            obj => anyhow::bail!("expected name, found {:?}", obj),
        }
    }

    fn assert_string(&mut self, obj: Object) -> PdfResult<String> {
        match obj {
            Object::String(s) => Ok(s),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_string(obj)
            }
            obj => anyhow::bail!("expected string, found {:?}", obj),
        }
    }

    fn assert_arr(&mut self, obj: Object<'a>) -> PdfResult<Vec<Object<'a>>> {
        match obj {
            Object::Array(a) => Ok(a),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_arr(obj)
            }
            obj => anyhow::bail!("expected array, found {:?}", obj),
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
            obj => anyhow::bail!("expected boolean, found {:?}", obj),
        }
    }

    fn assert_stream(&mut self, obj: Object<'a>) -> PdfResult<Stream<'a>> {
        match obj {
            Object::Stream(s) => Ok(s),
            Object::Reference(r) => {
                let obj = self.lex_object_from_reference(r)?;
                self.assert_stream(obj)
            }
            obj => anyhow::bail!("expected stream, found {:?}", obj),
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
