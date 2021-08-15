use crate::{error::PdfResult, resolve::Resolve, stream::Stream};

#[derive(Debug)]
pub(crate) struct ToUnicodeCmapStream<'a> {
    stream: Stream<'a>,
}

impl<'a> ToUnicodeCmapStream<'a> {
    const TYPE: &'static str = "CMap";

    pub fn from_stream(stream: Stream<'a>, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self { stream })
    }
}
