use std::fmt;

use crate::{
    data_structures::Rectangle,
    error::PdfResult,
    objects::{Dictionary, Name, Object, TypedReference},
    stream::Stream,
    FromObj, Resolve,
};

use super::embedded::{TrueTypeFontFile, Type1FontFile, Type3FontFile};

#[derive(Debug, FromObj)]
#[obj_type("FontDescriptor")]
pub struct FontDescriptor<'a> {
    /// The PostScript name of the font. This name shall be the same as the value of
    /// BaseFont in the font or CIDFont dictionary that refers to this font descriptor
    #[field("FontName")]
    font_name: Name,

    /// A byte string specifying the preferred font family name
    #[field("FontFamily")]
    font_family: Option<String>,

    /// The font stretch value
    ///
    /// The specific interpretation of this value varies from font to font
    #[field("FontStretch")]
    font_stretch: Option<FontStretch>,

    /// The weight (thickness) component of the fully-qualified font name or font
    /// specifier
    ///
    /// The specific interpretation of these values varies from font to font
    #[field("FontWeight")]
    font_weight: Option<f32>,

    /// A collection of flags defining various characteristics of the font
    #[field("Flags")]
    flags: FontDescriptorFlags,

    /// A rectangle, expressed in the glyph coordinate system, that shall specify
    /// the font bounding box. This should be the smallest rectangle enclosing the
    /// shape that would result if all of the glyphs of the font were placed with
    /// their origins coincident and then filled
    #[field("FontBBox")]
    font_bounding_box: Option<Rectangle>,

    /// The angle, expressed in degrees counterclockwise from the vertical, of the dominant
    /// vertical strokes of the font.
    ///
    /// EXAMPLE 4 The 9-o’clock position is 90 degrees, and the 3-o’clock position is –90 degrees.
    ///
    /// The value shall be negative for fonts that slope to the right, as almost all italic fonts do
    #[field("ItalicAngle")]
    italic_angle: f32,

    /// The maximum height above the baseline reached by glyphs in this font. The height of
    /// glyphs for accented characters shall be excluded
    #[field("Ascent")]
    ascent: Option<f32>,

    /// The maximum depth below the baseline reached by glyphs in this font
    ///
    /// The value shall be a negative number
    #[field("Descent")]
    descent: Option<f32>,

    /// The spacing between baselines of consecutive lines of text.
    ///
    /// Default value: 0
    #[field("Leading", default = 0.0)]
    leading: f32,

    /// The vertical coordinate of the top of flat capital letters, measured from the baseline
    #[field("CapHeight")]
    cap_height: Option<f32>,

    /// The font’s x height: the vertical coordinate of the top of flat nonascending lowercase
    /// letters (like the letter x), measured from the baseline, in fonts that have Latin characters
    ///
    /// Default value: 0
    #[field("XHeight", default = 0.0)]
    x_height: f32,

    /// The thickness, measured horizontally, of the dominant vertical stems of glyphs in the font
    #[field("StemV")]
    stem_v: Option<f32>,

    /// The thickness, measured vertically, of the dominant horizontal stems of glyphs in the font
    ///
    /// Default value: 0
    #[field("StemH", default = 0.0)]
    stem_h: f32,

    /// The average width of glyphs in the font
    ///
    /// Default value: 0
    #[field("AvgWidth", default = 0.0)]
    avg_width: f32,

    /// The maximum width of glyphs in the font
    ///
    /// Default value: 0
    #[field("MaxWidth", default = 0.0)]
    max_width: f32,

    /// The width to use for character codes whose widths are not specified in a font dictionary’s
    /// Widths array. This shall have a predictable effect only if all such codes map to glyphs whose
    /// actual widths are the same as the value of the MissingWidth entry
    ///
    /// Default value: 0
    #[field("MissingWidth", default = 0.0)]
    pub missing_width: f32,

    /// A stream containing a Type 1 font program
    #[field("FontFile")]
    pub font_file: Option<Type1FontFile<'a>>,

    /// A stream containing a TrueType font program
    #[field("FontFile2")]
    pub font_file_two: Option<TrueTypeFontFile<'a>>,

    /// A stream containing a font program whose format is specified by the Subtype entry in the
    /// stream dictionary
    #[field("FontFile3")]
    font_file_three: Option<Type3FontFile<'a>>,

    /// A string listing the character names defined in a font subset. The names in this string
    /// shall be in PDF syntax—that is, each name preceded by a slash (/). The names may appear in
    /// any order. The name .notdef shall be omitted; it shall exist in the font subset. If this entry
    /// is absent, the only indication of a font subset shall be the subset tag in the FontName entry
    ///
    /// Meaningful only in Type 1 fonts
    #[field("CharSet")]
    charset: Option<String>,

    /// A dictionary containing entries that describe the style of the glyphs in
    /// the font
    ///
    /// Meaningful only in CID fonts
    #[field("Style")]
    style: Option<Dictionary<'a>>,

    /// A name specifying the language of the font, which may be used for encodings
    /// where the language is not implied by the encoding itself.
    ///
    /// Meaningful only in CID fonts
    #[field("Lang")]
    lang: Option<Name>,

    /// A dictionary whose keys identify a class of glyphs in a CIDFont.
    ///
    /// Each value shall be a dictionary containing entries that shall override
    /// the corresponding values in the main font descriptor dictionary for that
    /// class of glyphs
    ///
    /// Meaningful only in CID fonts
    #[field("FD")]
    fd: Option<Dictionary<'a>>,

    /// A stream identifying which CIDs are present in the CIDFont file. If this
    /// entry is present, the CIDFont shall contain only a subset of the glyphs
    /// in the character collection defined by the CIDSystemInfo dictionary. If
    /// it is absent, the only indication of a CIDFont subset shall be the subset
    /// tag in the FontName entry
    ///
    /// The stream’s data shall be organized as a table of bits indexed by CID.
    /// The bits shall be stored in bytes with the high-order bit first. Each bit
    /// shall correspond to a CID. The most significant bit of the first byte
    /// shall correspond to CID 0, the next bit to CID 1, and so on.
    ///
    /// Meaningful only in CID fonts
    #[field("CIDSet")]
    cid_set: Option<TypedReference<'a, Stream<'a>>>,
}

#[derive(Debug)]
struct CidFontDescriptor<'a> {
    base: FontDescriptor<'a>,
}

// todo: derive FromObj for tuple structs
#[derive(Clone, Copy)]
struct FontDescriptorFlags(u32);

impl fmt::Debug for FontDescriptorFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

impl<'a> FromObj<'a> for FontDescriptorFlags {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(u32::from_obj(obj, resolver)?))
    }
}

impl FontDescriptorFlags {
    const FIXED_PITCH: u32 = 1 << 0;
    const SERIF: u32 = 1 << 1;
    const SYMBOLIC: u32 = 1 << 2;
    const SCRIPT: u32 = 1 << 3;
    const NON_SYMBOLIC: u32 = 1 << 5;
    const ITALIC: u32 = 1 << 6;
    const ALL_CAP: u32 = 1 << 16;
    const SMALL_CAP: u32 = 1 << 17;
    const FORCE_BOLD: u32 = 1 << 18;

    /// All glyphs have the same width (as opposed to proportional or variable-pitch
    /// fonts, which have different widths).
    pub const fn is_fixed_pitch(&self) -> bool {
        self.0 & Self::FIXED_PITCH != 0
    }

    /// Glyphs have serifs, which are short strokes drawn at an angle on the top and
    /// bottom of glyph stems. (Sans serif fonts do not have serifs.)
    pub const fn is_serif(&self) -> bool {
        self.0 & Self::SERIF != 0
    }

    /// Glyphs have serifs, which are short strokes drawn at an angle on the top and
    /// bottom of glyph stems. (Sans serif fonts do not have serifs.)
    pub const fn is_sans_serif(&self) -> bool {
        self.0 & Self::SERIF == 0
    }

    /// Font contains glyphs outside the Adobe standard Latin character set.
    ///
    /// This flag and the Nonsymbolic flag shall not both be set or both be clear
    pub const fn is_symbolic(&self) -> bool {
        self.0 & Self::SYMBOLIC != 0
    }

    /// Glyphs resemble cursive handwriting
    pub const fn is_script(&self) -> bool {
        self.0 & Self::SCRIPT != 0
    }

    /// Font uses the Adobe standard Latin character set or a subset of it
    pub const fn is_non_symbolic(&self) -> bool {
        self.0 & Self::NON_SYMBOLIC != 0
    }

    /// Glyphs have dominant vertical strokes that are slanted
    pub const fn is_italic(&self) -> bool {
        self.0 & Self::ITALIC != 0
    }

    /// Font contains no lowercase letters; typically used for display purposes,
    /// such as for titles or headlines
    pub const fn is_all_cap(&self) -> bool {
        self.0 & Self::ALL_CAP != 0
    }

    /// Font contains both uppercase and lowercase letters. The uppercase letters
    /// are similar to those in the regular version of the same typeface family.
    /// The glyphs for the lowercase letters have the same shapes as the corresponding
    /// uppercase letters, but they are sized and their proportions adjusted so that
    /// they have the same size and stroke weight as lowercase glyphs in the same typeface
    /// family
    pub const fn is_small_cap(&self) -> bool {
        self.0 & Self::SMALL_CAP != 0
    }
}

#[pdf_enum]
enum FontStretch {
    UltraCondensed = "UltraCondensed",
    ExtraCondensed = "ExtraCondensed",
    Condensed = "Condensed",
    SemiCondensed = "SemiCondensed",
    Normal = "Normal",
    SemiExpanded = "SemiExpanded",
    Expanded = "Expanded",
    ExtraExpanded = "ExtraExpanded",
    UltraExpanded = "UltraExpanded",
}
