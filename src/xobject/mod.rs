use crate::{error::PdfResult, objects::Dictionary,  stream::Stream, Resolve};

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

    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        dict.expect_type(Self::TYPE, resolver, false)?;

        let subtype = XObjectSubtype::from_str(&dict.expect_name("Subtype", resolver)?)?;

        Ok(match subtype {
            XObjectSubtype::PostScript => {
                XObject::PostScript(PostScriptXObject::from_stream(stream, resolver)?)
            }
            XObjectSubtype::Image => XObject::Image(ImageXObject::from_stream(stream, resolver)?),
            XObjectSubtype::Form => XObject::Form(FormXObject::from_stream(stream, resolver)?),
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenPrepressInterface;

impl OpenPrepressInterface {
    pub fn from_dict<'a>(_dict: Dictionary, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        todo!()
    }
}
