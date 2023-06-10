use crate::{error::PdfResult, filter::decode_stream, objects::Object, FromObj, Resolve};

use super::ColorSpace;

#[derive(Debug, Clone)]
pub struct IndexedColorSpace<'a> {
    pub base: ColorSpace<'a>,

    /// The hival parameter shall be an integer that specifies the maximum valid
    /// index value. The colour table shall be indexed by integers in the range 0
    /// to hival. hival shall be no greater than 255, which is the integer
    /// required to index a table with 8-bit index values.
    pub hival: u8,

    pub lookup: IndexedLookupTable,
}

#[derive(Debug, Clone)]
pub struct IndexedLookupTable {
    buffer: Vec<u8>,
}

impl<'a> FromObj<'a> for IndexedLookupTable {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let buffer = match resolver.resolve(obj)? {
            Object::String(s) => s.into_bytes(),
            Object::Stream(stream) => {
                decode_stream(&stream.stream, &stream.dict, resolver)?.into_owned()
            }
            obj => anyhow::bail!(
                "expected string or stream for indexed lookup table, got {:?}",
                obj
            ),
        };

        Ok(Self { buffer })
    }
}
