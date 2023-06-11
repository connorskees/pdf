use std::{
    borrow::Cow,
    io::{BufReader, Cursor},
};

pub struct DctDecoder<'a> {
    buffer: Cow<'a, [u8]>,
}

impl<'a> DctDecoder<'a> {
    pub fn new(buffer: Cow<'a, [u8]>) -> Self {
        Self { buffer }
    }

    pub fn decode(self) -> anyhow::Result<Vec<u8>> {
        let mut decoder = jpeg_decoder::Decoder::new(BufReader::new(Cursor::new(self.buffer)));

        Ok(decoder.decode()?)
    }
}

#[pdf_enum(Integer)]
pub enum ColorTransform {
    /// No transformation.
    None = 0,

    /// If the image has three colour components, RGB values shall be transformed to
    /// YUV before encoding and from YUV to RGB after decoding. If the image has
    /// four components, CMYK values shall be transformed to YUVK before encoding
    /// and from YUVK to CMYK after decoding. This option shall be ignored if the
    /// image has one or two colour components.
    Yuv = 1,
}
