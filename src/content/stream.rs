use std::fmt;

use crate::{
    error::{ParseError, PdfResult},
    filter::decode_stream,
    objects::{Object, ObjectType},
    resolve::Resolve,
    stream::Stream,
};

#[derive(Clone)]
pub(crate) struct ContentStream {
    pub combined_buffer: Vec<u8>,
}

impl fmt::Debug for ContentStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentStream")
            .field(
                "combined_buffer",
                &format!("[ {} bytes ]", self.combined_buffer.len()),
            )
            .finish()
    }
}

impl ContentStream {
    pub fn from_obj<'a>(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let streams = match resolver.resolve(obj)? {
            Object::Stream(stream) => vec![stream],
            Object::Array(arr) => arr
                .into_iter()
                .map(|obj| resolver.assert_stream(obj))
                .collect::<PdfResult<Vec<Stream>>>()?,
            _ => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Array, ObjectType::Stream],
                });
            }
        };

        let combined_buffer =
            streams
                .into_iter()
                .try_fold(Vec::new(), |mut init, stream| -> PdfResult<Vec<u8>> {
                    init.extend(
                        decode_stream(&stream.stream, &stream.dict, resolver)?
                            .iter()
                            .copied(),
                    );

                    Ok(init)
                })?;

        Ok(Self { combined_buffer })
    }
}
