use crate::{error::PdfResult, resolve::Resolve, stream::Stream, FromObj, objects::Object};

#[derive(Debug)]
pub(crate) struct ToUnicodeCmapStream<'a> {
    stream: Stream<'a>,
}

impl<'a> ToUnicodeCmapStream<'a> {
    const TYPE: &'static str = "CMap";
}

impl<'a> FromObj<'a> for ToUnicodeCmapStream<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut stream = resolver.assert_stream(obj)?;

        stream.dict.other.expect_type(Self::TYPE, resolver, false)?;

        Ok(Self { stream })
    }
}
