use crate::{
    error::{ParseError, PdfResult},
    objects::{Name, Object, ObjectType},
    stream::Stream,
    FromObj, Resolve,
};

use super::descriptor::FontDescriptor;

#[derive(Debug, FromObj)]
pub struct CidSystemInfo {
    /// A string identifying the issuer of the character collection
    ///
    /// For information about assigning a registry identifier, contact
    /// the Adobe Solutions Network or consult the ASN Web site
    #[field("Registry")]
    registry: String,

    /// A string that uniquely names the character collection within the
    /// specified registry
    #[field("Ordering")]
    ordering: String,

    /// The supplement number of the character collection. An original
    /// character collection has a supplement number of 0. Whenever additional
    /// CIDs are assigned in a character collection, the supplement number
    /// shall be increased. Supplements shall not alter the ordering of
    /// existing CIDs in the character collection. This value shall not
    /// be used in determining compatibility between character collections
    #[field("Supplement")]
    supplement: i32,
}

#[derive(Debug, FromObj)]
#[obj_type("Font")]
pub struct CidFontDictionary<'a> {
    /// The PostScript name of the CIDFont. For Type 0 CIDFonts, this shall be
    /// the value of the CIDFontName entry in the CIDFont program. For Type 2
    /// CIDFonts, it shall be derived the same way as for a simple TrueType font.
    /// In either case, the name may have a subset prefix if appropriate
    #[field("BaseFont")]
    base_font: Name,

    /// A dictionary containing entries that define the character collection of the
    /// CIDFont
    #[field("CIDSystemInfo")]
    cid_system_info: CidSystemInfo,

    /// A font descriptor describing the CIDFont’s default metrics other than its
    /// glyph widths
    #[field("FontDescriptor")]
    font_descriptor: FontDescriptor<'a>,

    /// The default width for glyphs in the CIDFont
    ///
    /// Default value: 1000 (defined in user units)
    #[field("DW", default = 1000)]
    dw: i32,

    /// A description of the widths for the glyphs in the CIDFont
    ///
    /// NOTE: The array’s elements have a variable format that can specify individual
    ///       widths for consecutive CIDs or one width for a range of CIDs
    ///
    /// Default value: none (the DW value shall be used for all glyphs)
    #[field("W")]
    w: Option<Vec<Object<'a>>>,

    /// An array of two numbers specifying the default metrics for vertical writing
    ///
    /// Default value: [880 −1000]
    #[field("DW2", default = [880.0, -1000.0])]
    dw2: [f32; 2],

    /// A description of the metrics for vertical writing for the glyphs in the CIDFont
    ///
    /// Default value: none (the DW2 value shall be used for all glyphs)
    #[field("W2")]
    w2: Option<Vec<Object<'a>>>,

    /// A specification of the mapping from CIDs to glyph indices. If the value is a
    /// stream, the bytes in the stream shall contain the mapping from CIDs to glyph
    /// indices: the glyph index for a particular CID value c shall be a 2-byte value
    /// stored in bytes 2 × c and 2 × c + 1, where the first byte shall be the high-order
    /// byte. If the value of CIDToGIDMap is a name, it shall be Identity, indicating that
    /// the mapping between CIDs and glyph indices is the identity mapping.
    ///
    /// Default value: Identity
    ///
    /// This entry may appear only in a Type 2 CIDFont whose associated TrueType font program
    /// is embedded in the PDF file
    #[field("CIDToGIDMap", default = CidToGidMap::Identity)]
    cid_to_gid_map: CidToGidMap<'a>,
}

#[derive(Debug)]
enum CidToGidMap<'a> {
    Identity,
    Stream(Stream<'a>),
}

impl<'a> FromObj<'a> for CidToGidMap<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match resolver.resolve(obj)? {
            Object::Name(ref name) if name == "Identity" => Self::Identity,
            Object::Stream(stream) => Self::Stream(stream),
            _ => {
                anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Name, ObjectType::Stream],
                });
            }
        })
    }
}
