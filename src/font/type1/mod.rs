use crate::{error::PdfResult, font::cmap::ToUnicodeCmapStream, objects::Dictionary, Resolve};

use super::{encoding::FontEncoding, BaseFontDict};

mod data;

/// A Type 1 font program is a stylized PostScript program that describes glyph
/// shapes. It uses a compact encoding for the glyph descriptions, and it includes
/// hint information that enables high-quality rendering even at small sizes and low
/// resolutions
#[derive(Debug)]
pub struct Type1Font<'a> {
    pub base: BaseFontDict<'a>,

    /// The PostScript name of the font. For Type 1 fonts, this is always the value
    /// of the FontName entry in the font program; for more information, see
    /// Section 5.2 of the PostScript Language Reference, Third Edition. The
    /// PostScript name of the font may be used to find the font program in the
    /// conforming reader or its environment. It is also the name that is used
    /// when printing to a PostScript output device
    pub base_font: String,

    /// A specification of the font's character encoding if different from its
    /// built-in encoding.
    ///
    /// The value of `encoding` shall be either the name of a predefined encoding
    /// (MacRomanEncoding, MacExpertEncoding, or WinAnsiEncoding) or an encoding
    /// dictionary that shall specify differences from the font's built-in
    /// encoding or from a specified predefined encoding.
    pub encoding: Option<FontEncoding>,

    /// A stream containing a CMap file that maps character codes to Unicode values
    pub to_unicode: Option<ToUnicodeCmapStream<'a>>,
}

impl<'a> Type1Font<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let base = BaseFontDict::from_dict(&mut dict, resolver)?;
        let base_font = dict.expect_name("BaseFont", resolver)?;
        let encoding = dict.get("Encoding", resolver)?;
        let to_unicode = dict.get("ToUnicode", resolver)?;

        Ok(Self {
            base,
            base_font,
            encoding,
            to_unicode,
        })
    }
}

/// The multiple master font format is an extension of the Type 1 font format that
/// allows the generation of a wide variety of typeface styles from a single font
/// program. This is accomplished through the presence of various design dimensions
/// in the font
#[derive(Debug)]
pub struct MmType1Font<'a> {
    type1: Type1Font<'a>,
}

impl<'a> MmType1Font<'a> {
    pub fn from_dict(dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self {
            type1: Type1Font::from_dict(dict, resolver)?,
        })
    }
}
