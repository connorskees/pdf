use std::collections::HashMap;

use crate::{
    data_structures::{Matrix, Rectangle},
    error::PdfResult,
    font::cmap::ToUnicodeCmapStream,
    objects::Dictionary,
    resources::Resources,
    stream::Stream,
    FromObj, Resolve,
};

use super::{encoding::FontEncoding, BaseFontDict};

/// Type 3 fonts differ from the other fonts supported by PDF. A Type 3 font dictionary
/// defines the font; font dictionaries for other fonts simply contain information about
/// the font and refer to a separate font program for the actual glyph descriptions. In
/// Type 3 fonts, glyphs shall be defined by streams of PDF graphics operators. These
/// streams shall be associated with glyph names. A separate encoding entry shall map
/// character codes to the appropriate glyph names for the glyphs
#[derive(Debug, Clone)]
pub struct Type3Font<'a> {
    pub base: BaseFontDict<'a>,

    /// A rectangle expressed in the glyph coordinate system, specifying the font
    /// bounding box. This is the smallest rectangle enclosing the shape that would
    /// result if all of the glyphs of the font were placed with their origins coincident
    /// and then filled.
    ///
    /// If all four elements of the rectangle are zero, a conforming reader shall make
    /// no assumptions about glyph sizes based on the font bounding box. If any element
    /// is nonzero, the font bounding box shall be accurate. If any glyph’s marks fall
    /// outside this bounding box, incorrect behavior may result
    font_bounding_box: Rectangle,

    /// An array of six numbers specifying the font matrix, mapping glyph space to text space.
    ///
    /// NOTE: A common practice is to define glyphs in terms of a 1000-unit glyph coordinate
    ///       system, in which case the font matrix is [0.001 0 0 0.001 0 0]
    pub font_matrix: Matrix,

    /// A dictionary in which each key shall be a glyph name and the value associated
    /// with that key shall be a content stream that constructs and paints the glyph for
    /// that character. The stream shall include as its first operator either d0 or d1,
    /// followed by operators describing one or more graphics objects, which may include
    /// path, text, or image objects
    pub char_procs: HashMap<String, Stream<'a>>,

    /// An encoding dictionary whose Differences array shall specify the complete
    /// character encoding for this font
    pub encoding: FontEncoding,

    /// A list of the named resources, such as fonts and images, required by the glyph
    /// descriptions in this font. If any glyph descriptions refer to named resources but this
    /// dictionary is absent, the names shall be looked up in the resource dictionary of the
    /// page on which the font is used
    resources: Option<Resources<'a>>,

    /// A stream containing a CMap file that maps character codes to Unicode values
    to_unicode: Option<ToUnicodeCmapStream<'a>>,
}

impl<'a> Type3Font<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let base = BaseFontDict::from_dict(&mut dict, resolver)?;
        let font_bounding_box = dict.expect::<Rectangle>("FontBBox", resolver)?;
        let font_matrix = dict
            .get::<Matrix>("Matrix", resolver)?
            .unwrap_or_else(|| Matrix::new(0.001, 0.0, 0.0, 0.001, 0.0, 0.0));
        let char_procs = dict
            .expect_dict("CharProcs", resolver)?
            .entries()
            .map(|(key, obj)| Ok((key, resolver.assert_stream(obj)?)))
            .collect::<PdfResult<_>>()?;
        let encoding = FontEncoding::from_obj(dict.expect_object("Encoding", resolver)?, resolver)?;
        let resources = dict.get("Resources", resolver)?;
        let to_unicode = dict.get("ToUnicode", resolver)?;

        Ok(Self {
            base,
            font_bounding_box,
            font_matrix,
            char_procs,
            encoding,
            resources,
            to_unicode,
        })
    }
}
