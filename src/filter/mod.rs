use std::borrow::Cow;

use crate::{
    error::PdfResult, filter::dct::DctDecoder, objects::Dictionary, pdf_enum, stream::StreamDict,
    Resolve,
};

use flate::{FlateDecoder, FlateDecoderParams};

pub mod ascii;
pub mod dct;
pub mod flate;

pub(crate) fn decode_stream<'a>(
    stream: &'a [u8],
    stream_dict: &StreamDict,
    resolver: &mut dyn Resolve,
) -> PdfResult<Cow<'a, [u8]>> {
    if let Some(filters) = &stream_dict.filter {
        if filters.is_empty() {
            return Ok(Cow::Borrowed(stream));
        }

        let mut stream = stream.to_vec();

        for (idx, filter) in filters.into_iter().enumerate() {
            let decode_params = stream_dict
                .decode_parms
                .clone()
                .and_then(|params| params.get(idx).cloned())
                .unwrap_or_else(Dictionary::empty);

            match filter {
                FilterKind::AsciiHex => {
                    stream = ascii::decode_ascii_hex(&stream);
                }
                FilterKind::Ascii85 => {
                    stream = ascii::decode_ascii_85(&stream);
                }
                FilterKind::Lzw => todo!(),
                FilterKind::Flate => {
                    let decoder_params = FlateDecoderParams::from_dict(decode_params, resolver)?;

                    stream = FlateDecoder::new(Cow::Owned(stream), decoder_params).decode();
                }
                FilterKind::RunLength => todo!(),
                FilterKind::CcittFax => todo!(),
                FilterKind::Jbig2 => todo!(),
                FilterKind::Dct => stream = DctDecoder::new(Cow::Owned(stream)).decode(),
                FilterKind::Jpx => todo!(),
                FilterKind::Crypt => todo!(),
            }
        }

        return Ok(Cow::Owned(stream));
    }

    Ok(Cow::Borrowed(stream))
}

pdf_enum!(
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum FilterKind {
        /// Decodes data encoded in an ASCII hexadecimal representation, reproducing
        /// the original binary data
        AsciiHex = "ASCIIHexDecode",

        /// Decodes data encoded in an ASCII base-85 representation, reproducing the
        /// original binary data
        Ascii85 = "ASCII85Decode",

        /// Decompresses data encoded using the LZW (Lempel-ZivWelch) adaptive compression
        /// method, reproducing the original text or binary data
        Lzw = "LZWDecode",

        /// Decompresses data encoded using the zlib/deflate compression method,
        /// reproducing the original text or binary data
        Flate = "FlateDecode",

        /// Decompresses data encoded using a byte-oriented run-length encoding algorithm,
        /// reproducing the original text or binary data (typically monochrome image data,
        /// or any data that contains frequent long runs of a single byte value)
        RunLength = "RunLengthDecode",

        /// Decompresses data encoded using the CCITT facsimile standard, reproducing
        /// the original data (typically monochrome image data at 1 bit per pixel)
        CcittFax = "CCITTFaxDecode",

        /// Decompresses data encoded using the JBIG2 standard, reproducing the original
        /// monochrome (1 bit per pixel) image data (or an approximation of that data)
        Jbig2 = "JBIG2Decode",

        /// Decompresses data encoded using a DCT (discrete cosine transform) technique
        /// based on the JPEG standard, reproducing image sample data that approximates
        /// the original data
        Dct = "DCTDecode",

        /// Decompresses data encoded using the waveletbased JPEG2000 standard, reproducing
        /// the original image data
        Jpx = "JPXDecode",

        /// Decrypts data encrypted by a security handler, reproducing the data as it
        /// was before encryption
        Crypt = "Crypt",
    }
);
