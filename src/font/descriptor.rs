use std::fmt;

use crate::{data_structures::Rectangle, error::PdfResult, objects::Dictionary, pdf_enum, Resolve};

use super::embedded::{TrueTypeFontFile, Type1FontFile, Type3FontFile};

#[derive(Debug)]
pub struct FontDescriptor {
    /// The PostScript name of the font. This name shall be the same as the value of
    /// BaseFont in the font or CIDFont dictionary that refers to this font descriptor
    font_name: String,

    /// A byte string specifying the preferred font family name
    font_family: Option<String>,

    /// The font stretch value
    ///
    /// The specific interpretation of this value varies from font to font
    font_stretch: Option<FontStretch>,

    /// The weight (thickness) component of the fully-qualified font name or font
    /// specifier
    ///
    /// The specific interpretation of these values varies from font to font
    ///
    /// N.B.: Although the possible values of font weight are all positive integers, it is possible
    /// for this value to be a float
    font_weight: Option<FontWeight>,

    /// A collection of flags defining various characteristics of the font
    flags: FontDescriptorFlags,

    /// A rectangle, expressed in the glyph coordinate system, that shall specify
    /// the font bounding box. This should be the smallest rectangle enclosing the
    /// shape that would result if all of the glyphs of the font were placed with
    /// their origins coincident and then filled
    font_bounding_box: Option<Rectangle>,

    /// The angle, expressed in degrees counterclockwise from the vertical, of the dominant
    /// vertical strokes of the font.
    ///
    /// EXAMPLE 4 The 9-o’clock position is 90 degrees, and the 3-o’clock position is –90 degrees.
    ///
    /// The value shall be negative for fonts that slope to the right, as almost all italic fonts do
    italic_angle: f32,

    /// The maximum height above the baseline reached by glyphs in this font. The height of
    /// glyphs for accented characters shall be excluded
    ascent: Option<f32>,

    /// The maximum depth below the baseline reached by glyphs in this font
    ///
    /// The value shall be a negative number
    descent: Option<f32>,

    /// The spacing between baselines of consecutive lines of text.
    ///
    /// Default value: 0
    leading: f32,

    /// The vertical coordinate of the top of flat capital letters, measured from the baseline
    cap_height: Option<f32>,

    /// The font’s x height: the vertical coordinate of the top of flat nonascending lowercase
    /// letters (like the letter x), measured from the baseline, in fonts that have Latin characters
    ///
    /// Default value: 0
    x_height: f32,

    /// The thickness, measured horizontally, of the dominant vertical stems of glyphs in the font
    stem_v: Option<f32>,

    /// The thickness, measured vertically, of the dominant horizontal stems of glyphs in the font
    ///
    /// Default value: 0
    stem_h: f32,

    /// The average width of glyphs in the font
    ///
    /// Default value: 0
    avg_width: f32,

    /// The maximum width of glyphs in the font
    ///
    /// Default value: 0
    max_width: f32,

    /// The width to use for character codes whose widths are not specified in a font dictionary’s
    /// Widths array. This shall have a predictable effect only if all such codes map to glyphs whose
    /// actual widths are the same as the value of the MissingWidth entry
    ///
    /// Default value: 0
    missing_width: f32,

    /// A stream containing a Type 1 font program
    font_file: Option<Type1FontFile>,

    /// A stream containing a TrueType font program
    font_file_two: Option<TrueTypeFontFile>,

    /// A stream containing a font program whose format is specified by the Subtype entry in the
    /// stream dictionary
    font_file_three: Option<Type3FontFile>,

    /// A string listing the character names defined in a font subset. The names in this string
    /// shall be in PDF syntax—that is, each name preceded by a slash (/). The names may appear in
    /// any order. The name .notdef shall be omitted; it shall exist in the font subset. If this entry
    /// is absent, the only indication of a font subset shall be the subset tag in the FontName entry
    ///
    /// Meaningful only in Type 1 fonts
    charset: Option<String>,
}

#[derive(Debug)]
struct CidFontDescriptor {
    base: FontDescriptor,
}

impl FontDescriptor {
    const TYPE: &'static str = "FontDescriptor";

    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, true)?;

        let font_name = dict.expect_name("FontName", resolver)?;
        let font_family = dict.get_string("FontFamily", resolver)?;
        let font_stretch = dict
            .get_name("FontStretch", resolver)?
            .as_deref()
            .map(FontStretch::from_str)
            .transpose()?;
        let font_weight = dict
            .get_number("FontWeight", resolver)?
            .map(|n| FontWeight::from_integer(n as i32))
            .transpose()?;
        let flags = FontDescriptorFlags(dict.expect_unsigned_integer("Flags", resolver)?);
        let font_bounding_box = dict.get_rectangle("FontBBox", resolver)?;
        let italic_angle = dict.expect_number("ItalicAngle", resolver)?;

        let ascent = dict.get_number("Ascent", resolver)?;
        let descent = dict.get_number("Descent", resolver)?;
        let leading = dict.get_number("Leading", resolver)?.unwrap_or(0.0);
        let cap_height = dict.get_number("CapHeight", resolver)?;
        let x_height = dict.get_number("XHeight", resolver)?.unwrap_or(0.0);
        let stem_v = dict.get_number("StemV", resolver)?;
        let stem_h = dict.get_number("StemH", resolver)?.unwrap_or(0.0);
        let avg_width = dict.get_number("AvgWidth", resolver)?.unwrap_or(0.0);
        let max_width = dict.get_number("MaxWidth", resolver)?.unwrap_or(0.0);
        let missing_width = dict.get_number("MissingWidth", resolver)?.unwrap_or(0.0);
        let font_file = dict
            .get_stream("FontFile", resolver)?
            .map(|stream| Type1FontFile::from_stream(stream, resolver))
            .transpose()?;
        let font_file_two = dict
            .get_stream("FontFile2", resolver)?
            .map(|stream| TrueTypeFontFile::from_stream(stream, resolver))
            .transpose()?;
        let font_file_three = dict
            .get_stream("FontFile3", resolver)?
            .map(|stream| Type3FontFile::from_stream(stream, resolver))
            .transpose()?;

        let charset = dict.get_string("CharSet", resolver)?;

        Ok(Self {
            font_name,
            font_family,
            font_stretch,
            font_weight,
            flags,
            font_bounding_box,
            italic_angle,
            ascent,
            descent,
            leading,
            cap_height,
            x_height,
            stem_v,
            stem_h,
            avg_width,
            max_width,
            missing_width,
            font_file,
            font_file_two,
            font_file_three,
            charset,
        })
    }
}

#[derive(Clone, Copy)]
struct FontDescriptorFlags(u32);

impl fmt::Debug for FontDescriptorFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:b}", self.0)
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

pdf_enum!(
    int
    #[derive(Debug)]
    enum FontWeight {
        OneHundred = 100,
        TwoHundred = 200,
        ThreeHundred = 300,

        /// Normal
        FourHundred = 400,

        FiveHundred = 500,
        SixHundred = 600,

        /// Bold
        SevenHundred = 700,

        EightHundred = 800,
        NineHundred = 900,
    }
);

pdf_enum!(
    #[derive(Debug)]
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
);
