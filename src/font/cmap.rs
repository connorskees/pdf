use crate::{error::PdfResult, resolve::Resolve, stream::Stream};

#[derive(Debug)]
pub(crate) struct ToUnicodeCmapStream {
    stream: Stream,
}

impl ToUnicodeCmapStream {
    const TYPE: &'static str = "CMap";

    pub fn from_stream(stream: Stream, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        Ok(Self { stream })
    }
}
