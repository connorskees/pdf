use type0::Type0Font;

use crate::{error::PdfResult, objects::Dictionary, pdf_enum, Resolve};

use self::{
    descriptor::FontDescriptor,
    true_type::TrueTypeFont,
    type1::{MmType1Font, Type1Font},
    type3::Type3Font,
};

mod cid;
mod cid_font_type0;
mod cid_font_type2;
mod descriptor;
mod embedded;
mod encoding;
mod true_type;
mod type0;
mod type1;
mod type3;

#[derive(Debug)]
pub enum Font {
    Type1(Type1Font),
    MmType1(MmType1Font),
    TrueType(TrueTypeFont),
    Type3(Type3Font),
    Type0(Type0Font),
}

impl Font {
    const TYPE: &'static str = "Font";

    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, true)?;

        let subtype = FontSubtype::from_str(&dict.expect_name("Subtype", resolver)?)?;

        Ok(match subtype {
            FontSubtype::Type1 => Self::Type1(Type1Font::from_dict(dict, resolver)?),
            FontSubtype::MmType1 => Self::MmType1(MmType1Font::from_dict(dict, resolver)?),
            FontSubtype::Type3 => Self::Type3(Type3Font::from_dict(dict, resolver)?),
            FontSubtype::TrueType => Self::TrueType(TrueTypeFont::from_dict(dict, resolver)?),
            FontSubtype::Type0 => Self::Type0(Type0Font::from_dict(dict, resolver)?),
            _ => todo!("unimplemented font subtype: {:?}\n{:#?}", subtype, dict),
        })
    }
}

#[derive(Debug)]
pub struct BaseFontDict {
    /// The name by which this font is referenced in the Font subdictionary
    /// of the current resource dictionary.
    ///
    /// This entry is obsolete and should not be used.
    name: Option<String>,

    /// The first character code defined in the font's Widths array.
    ///
    /// Beginning with PDF 1.5, the special treatment given to the standard 14 fonts
    /// is deprecated. Conforming writers should represent all fonts using a complete
    /// font descriptor. For backwards capability, conforming readers shall
    /// still provide the special treatment identified for the standard 14 fonts.
    ///
    /// Required except for the standard 14 fonts
    first_char: u32,

    /// The last character code defined in the font's Widths array
    last_char: u32,

    /// An array of (`last_char` - `first_char` + 1) widths, each element being the
    /// glyph width for the character code that equals `first_char` plus the array
    /// index. For character codes outside the range `first_char` to `last_char`, the
    /// value of MissingWidth from the FontDescriptor entry for this font shall be used.
    ///
    /// The glyph widths shall be measured in units in which 1000 units correspond to 1
    /// unit in text space. These widths shall be consistent with the actual widths given
    /// in the font program. For more information on glyph widths and other glyph metrics
    widths: Vec<f32>,

    /// A font descriptor describing the font's metrics other than its glyph widths.
    ///
    /// For the standard 14 fonts, the entries `first_char`, `last_char`, `widths`, and
    /// `font_descriptor` shall either all be present or all be absent. Ordinarily, these
    /// dictionary keys may be absent; specifying them enables a standard font to be overridden.
    font_descriptor: FontDescriptor,
}

impl BaseFontDict {
    pub fn from_dict(dict: &mut Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let name = dict.get_name("Name", resolver)?;
        let first_char = dict.expect_unsigned_integer("FirstChar", resolver)?;
        let last_char = dict.expect_unsigned_integer("LastChar", resolver)?;
        let widths = dict
            .expect_arr("Widths", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;
        let font_descriptor =
            FontDescriptor::from_dict(dict.expect_dict("FontDescriptor", resolver)?, resolver)?;

        Ok(Self {
            name,
            first_char,
            last_char,
            widths,
            font_descriptor,
        })
    }
}

pdf_enum!(
    #[derive(Debug, Clone, Copy)]
    enum FontSubtype {
        /// A composite font -- a font composed of glyphs from a descendant CIDFont
        Type0 = "Type0",

        /// A font that defines glyph shapes using Type 1 font technology
        Type1 = "Type1",

        /// A multiple master font -- an extension of the Type 1 font that allows
        /// the generation of a wide variety of typeface styles from a single font
        MmType1 = "MMType1",

        /// A font that defines glyphs with streams of PDF graphics operators
        Type3 = "Type3",

        /// A font based on the TrueType font format
        TrueType = "TrueType",

        /// A CIDFont whose glyph descriptions are based on Type 1 font technology
        CidFontType0 = "CIDFontType0",

        /// A CIDFont whose glyph descriptions are based on TrueType font technology
        CidFontType2 = "CIDFontType2",
    }
);
