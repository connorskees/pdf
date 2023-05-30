use crate::{error::PdfResult, objects::Object, FromObj, Resolve};

pub use self::{form::FormXObject, image::ImageXObject, postscript::PostScriptXObject};

mod form;
mod image;
mod postscript;
mod reference;

/// An external object (commonly called an XObject) is a graphics object
/// whose contents are defined by a self-contained stream, separate from the
/// content stream in which it is used
#[derive(Debug, Clone)]
pub enum XObject<'a> {
    Image(ImageXObject<'a>),
    Form(FormXObject<'a>),
    PostScript(PostScriptXObject<'a>),
}

#[pdf_enum]
enum XObjectSubtype {
    PostScript = "PS",
    Image = "Image",
    Form = "Form",
}

impl<'a> XObject<'a> {
    const TYPE: &'static str = "XObject";
}

impl<'a> FromObj<'a> for XObject<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut stream = resolver.assert_stream(obj)?;
        let dict = &mut stream.dict.other;

        dict.expect_type(Self::TYPE, resolver, false)?;

        let subtype = XObjectSubtype::from_str(&dict.expect_name("Subtype", resolver)?)?;

        Ok(match subtype {
            XObjectSubtype::PostScript => XObject::PostScript(PostScriptXObject::from_obj(
                Object::Stream(stream),
                resolver,
            )?),
            XObjectSubtype::Image => {
                XObject::Image(ImageXObject::from_obj(Object::Stream(stream), resolver)?)
            }
            XObjectSubtype::Form => {
                XObject::Form(FormXObject::from_obj(Object::Stream(stream), resolver)?)
            }
        })
    }
}

#[derive(Debug, Clone, FromObj)]
pub struct OpenPrepressInterface;
