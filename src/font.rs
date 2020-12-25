use crate::{
    objects::{Dictionary, Reference},
    stream::Stream,
};

enum Font {
    Type1(Type1Font),
}

struct Type1Font {
    /// The PostScript name of the font. For Type 1 fonts, this is
    /// always the value of the FontName entry in the font program; for more
    /// information, see Section 5.2 of the PostScript Language Reference,
    /// Third Edition. The PostScript name of the font may be used to find the
    /// font program in the conforming reader or its environment. It is also the
    /// name that is used when printing to a PostScript output device
    base_font: String,

    /// The first character code defined in the font's Widths array.
    ///
    /// Beginning with PDF 1.5, the special treatment given to the standard 14 fonts
    /// is deprecated. Conforming writers should represent all fonts using a complete
    /// font descriptor. For backwards capability, conforming readers shall
    /// still provide the special treatment identified for the standard 14 fonts.
    ///
    /// Required except for the standard 14 fonts
    first_char: i32,

    /// The last character code defined in the font's Widths array
    last_char: i32,

    /// An array of (`last_char` − `first_char` + 1) widths, each element being the
    /// glyph width for the character code that equals `first_char` plus the array
    /// index. For character codes outside the range `first_char` to `last_char`, the
    /// value of MissingWidth from the FontDescriptor entry for this font shall be used.
    ///
    /// The glyph widths shall be measured in units in which 1000 units correspond to 1
    /// unit in text space. These widths shall be consistent with the actual widths given
    /// in the font program. For more information on glyph widths and other glyph metrics
    widths: Vec<i32>,

    /// A font descriptor describing the font's metrics other than its glyph widths.
    ///
    /// For the standard 14 fonts, the entries `first_char`, `last_char`, `widths`, and
    /// `font_descriptor` shall either all be present or all be absent. Ordinarily, these
    /// dictionary keys may be absent; specifying them enables a standard font to be overridden.
    font_descriptor: Reference,

    /// A specification of the font's character encoding if different from its built-in encoding.
    ///
    /// The value of `encoding` shall be either the name of a predefined encoding (MacRomanEncoding,
    /// MacExpertEncoding, or WinAnsiEncoding, as described in Annex D) or an encoding dictionary
    /// that shall specify differences from the font's built-in encoding or from a specified predefined
    /// encoding.
    encoding: FontEncoding,

    /// A stream containing a CMap file that maps character codes to Unicode values
    to_unicode: Option<Stream>,
}

enum FontEncoding {
    /// Mac OS standard encoding for Latin text in Western writing systems.
    ///
    /// Conforming readers shall have a predefined encoding named MacRomanEncoding that may be used with
    /// both Type 1 and TrueType fonts.
    MacRomanEncoding,

    /// An encoding for use with expert fonts—ones containing the expert character set.
    ///
    /// Conforming readers shall have a predefined encoding named MacExpertEncoding. Despite its
    /// name, it is not a platform specific encoding; however, only certain fonts have the
    /// appropriate character set for use with this encoding. No such fonts are among the
    /// standard 14 predefined fonts.
    MacExpertEncoding,

    /// Windows Code Page 1252, often called the "Windows ANSI" encoding.
    ///
    /// This is the standard Windows encoding for Latin text in Western writing systems. Conforming
    /// readers shall have a predefined encoding named WinAnsiEncoding that may be used with both
    /// Type 1 and TrueType fonts.
    WinAnsiEncoding,
    Dictionary(Dictionary),
}

enum FontSubtype {
    /// A composite font—a font composed of glyphs from a descendant CIDFont
    Type0,

    /// A font that defines glyph shapes using Type 1 font technology
    Type1,

    /// A multiple master font—an extension of the Type 1 font that allows
    /// the generation of a wide variety of typeface styles from a single font
    MmType1,

    /// A font that defines glyphs with streams of PDF graphics operators
    Type3,

    /// A font based on the TrueType font format
    TrueType,

    /// A CIDFont whose glyph descriptions are based on Type 1 font technology
    CidFontType0,

    /// A CIDFont whose glyph descriptions are based on TrueType font technology
    CidFontType2,
}
