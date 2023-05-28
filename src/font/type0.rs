use crate::{
    error::{ParseError, PdfResult},
    objects::{Name, Object, ObjectType},
    stream::Stream,
    FromObj, Resolve,
};

use super::{cid::CidFontDictionary, cjk::PredefinedCjkCmapName, cmap::ToUnicodeCmapStream};

#[derive(Debug)]
enum Type0FontEncoding<'a> {
    Predefined(PredefinedCjkCmapName),
    Stream(Stream<'a>),
}

impl<'a> FromObj<'a> for Type0FontEncoding<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        match resolver.resolve(obj)? {
            Object::Name(name) => Ok(Self::Predefined(PredefinedCjkCmapName::from_str(&name)?)),
            Object::Stream(stream) => Ok(Self::Stream(stream)),
            _ => anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                expected: &[ObjectType::Stream, ObjectType::Name],
            }),
        }
    }
}

/// A composite font, also called a Type 0 font, is one whose glyphs are obtained
/// from a fontlike object called a CIDFont. A composite font shall be represented
/// by a font dictionary whose Subtype value is Type0. The Type 0 font is known
/// as the root font, and its associated CIDFont is called its descendant.
#[derive(Debug, FromObj)]
pub struct Type0Font<'a> {
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
    #[field("BaseFont")]
    base_font: Name,

    /// The name of a predefined CMap, or a stream containing a CMap that
    /// maps character codes to font numbers and CIDs. If the descendant is
    /// a Type 2 CIDFont whose associated TrueType font program is not embedded
    /// in the PDF file, the Encoding entry shall be a predefined CMap name
    #[field("Encoding")]
    encoding: Type0FontEncoding<'a>,

    /// A one-element array specifying the CIDFont dictionary that is the descendant of this Type 0 font
    #[field("DescendantFonts")]
    descendant_fonts: [CidFontDictionary<'a>; 1],

    /// A stream containing a CMap file that maps character codes to Unicode values
    #[field("ToUnicode")]
    to_unicode: Option<ToUnicodeCmapStream<'a>>,
}
