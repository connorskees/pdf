use crate::{catalog::assert_len, error::PdfResult, objects::Dictionary, stream::Stream, Resolve};

use super::{cid::CidFontDictionary, encoding::FontEncoding};

#[derive(Debug)]
pub struct Type0Font {
    /// The name of the font. If the descendant is a Type 0 CIDFont, this
    /// name should be the concatenation of the CIDFont’s BaseFont name,
    /// a hyphen, and the CMap name given in the Encoding entry (or the
    /// CMapName entry in the CMap). If the descendant is a Type 2 CIDFont,
    /// this name should be the same as the CIDFont’s BaseFont name
    ///
    /// NOTE: In principle, this is an arbitrary name, since there is no
    ///       font program associated directly with a Type 0 font dictionary.
    ///       The conventions described here ensure maximum compatibility with
    ///       existing readers
    base_font: String,

    /// The name of a predefined CMap, or a stream containing a CMap that
    /// maps character codes to font numbers and CIDs. If the descendant is
    /// a Type 2 CIDFont whose associated TrueType font program is not embedded
    /// in the PDF file, the Encoding entry shall be a predefined CMap name
    encoding: FontEncoding,

    /// A one-element array specifying the CIDFont dictionary that is the descendant of this Type 0 font
    descendant_fonts: CidFontDictionary,

    /// A stream containing a CMap file that maps character codes to Unicode values
    to_unicode: Option<Stream>,
}

impl Type0Font {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let base_font = dict.expect_name("BaseFont", resolver)?;
        let encoding = FontEncoding::from_obj(dict.expect_object("Encoding", resolver)?, resolver)?;
        let descendant_fonts = {
            let mut arr = dict.expect_arr("DescendantFonts", resolver)?;

            assert_len(&arr, 1)?;

            CidFontDictionary::from_dict(resolver.assert_dict(arr.pop().unwrap())?, resolver)
        }?;

        let to_unicode = dict.get_stream("ToUnicode", resolver)?;

        assert!(to_unicode.is_none(), "cmap?");

        Ok(Self {
            base_font,
            encoding,
            descendant_fonts,
            to_unicode,
        })
    }
}
