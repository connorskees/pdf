use crate::{error::PdfResult, stream::Stream, Resolve};

#[derive(Debug)]
pub struct PostScriptXObject {
    stream: Stream,

    /// A stream whose contents shall be used in place of the PostScript XObject’s stream
    /// when the target PostScript interpreter is known to support only LanguageLevel 1
    level_one: Option<Stream>,
}

impl PostScriptXObject {
    pub fn from_stream(mut stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let level_one = dict.get_stream("Level1", resolver)?;

        Ok(Self { stream, level_one })
    }
}
