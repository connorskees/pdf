use crate::{error::PdfResult, stream::Stream, Resolve};

#[derive(Debug, Clone)]
pub struct PostScriptXObject<'a> {
    stream: Stream<'a>,

    /// A stream whose contents shall be used in place of the PostScript XObject's stream
    /// when the target PostScript interpreter is known to support only LanguageLevel 1
    level_one: Option<Stream<'a>>,
}

impl<'a> PostScriptXObject<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let level_one = dict.get_stream("Level1", resolver)?;

        Ok(Self { stream, level_one })
    }
}
