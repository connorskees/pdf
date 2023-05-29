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
