use type0::Type0Font;

use crate::{error::PdfResult, objects::Dictionary, pdf_enum, Resolve};

pub use self::{
    descriptor::FontDescriptor,
    glyph::Glyph,
    true_type::TrueTypeFont,
    type1::{MmType1Font, Type1Font},
    type3::Type3Font,
};

mod cid;
mod cid_font_type0;
mod cid_font_type2;
mod cjk;
mod cmap;
mod descriptor;
mod embedded;
mod encoding;
mod glyph;
pub mod true_type;
mod type0;
mod type1;
mod type3;

#[derive(Debug)]
pub enum Font<'a> {
    Type1(Type1Font<'a>),
    MmType1(MmType1Font<'a>),
    TrueType(TrueTypeFont<'a>),
    Type3(Type3Font<'a>),
    Type0(Type0Font<'a>),
}

impl<'a> Font<'a> {
    const TYPE: &'static str = "Font";

    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
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
pub struct BaseFontDict<'a> {
    /// The name by which this font is referenced in the Font subdictionary
    /// of the current resource dictionary.
    ///
    /// This entry is obsolete and should not be used.
    name: Option<String>,

    pub widths: Option<Widths>,

    /// A font descriptor describing the font's metrics other than its glyph widths.
    ///
    /// For the standard 14 fonts, the entries `first_char`, `last_char`, `widths`, and
    /// `font_descriptor` shall either all be present or all be absent. Ordinarily, these
    /// dictionary keys may be absent; specifying them enables a standard font to be overridden.
    pub font_descriptor: Option<FontDescriptor<'a>>,
}

impl<'a> BaseFontDict<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let name = dict.get_name("Name", resolver)?;
        let first_char = dict.get_unsigned_integer("FirstChar", resolver)?;
        let last_char = dict.get_unsigned_integer("LastChar", resolver)?;
        let widths = dict
            .get_arr("Widths", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?;
        let font_descriptor = dict
            .get_dict("FontDescriptor", resolver)?
            .map(|dict| FontDescriptor::from_dict(dict, resolver))
            .transpose()?;

        let widths = Widths::new(
            widths,
            first_char,
            last_char,
            font_descriptor
                .as_ref()
                .map(|descriptor| descriptor.missing_width)
                .unwrap_or(0.0),
        );

        Ok(Self {
            name,
            widths,
            font_descriptor,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Widths {
    /// An array of (`last_char` - `first_char` + 1) widths, each element being the
    /// glyph width for the character code that equals `first_char` plus the array
    /// index. For character codes outside the range `first_char` to `last_char`, the
    /// value of MissingWidth from the FontDescriptor entry for this font shall be used.
    ///
    /// The glyph widths shall be measured in units in which 1000 units correspond to 1
    /// unit in text space. These widths shall be consistent with the actual widths given
    /// in the font program.
    widths: Vec<f32>,

    missing_width: f32,

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
}

impl Widths {
    pub fn new(
        widths: Option<Vec<f32>>,
        first_char: Option<u32>,
        last_char: Option<u32>,
        missing_width: f32,
    ) -> Option<Self> {
        Some(Self {
            widths: widths?,
            first_char: first_char?,
            last_char: last_char?,
            missing_width,
        })
    }

    pub fn get(&self, codepoint: u32) -> f32 {
        if codepoint < self.first_char {
            return self.missing_width / 1000.0;
        }

        self.widths
            .get((codepoint - self.first_char) as usize)
            .copied()
            .unwrap_or(self.missing_width)
            / 1000.0
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
