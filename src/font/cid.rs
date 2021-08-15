use std::convert::TryInto;

use crate::{
    catalog::assert_len,
    error::PdfResult,
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    Resolve,
};

use super::descriptor::FontDescriptor;

#[derive(Debug)]
pub struct CidSystemInfo {
    /// A string identifying the issuer of the character collection
    ///
    /// For information about assigning a registry identifier, contact
    /// the Adobe Solutions Network or consult the ASN Web site
    registry: String,

    /// A string that uniquely names the character collection within the
    /// specified registry
    ordering: String,

    /// The supplement number of the character collection. An original
    /// character collection has a supplement number of 0. Whenever additional
    /// CIDs are assigned in a character collection, the supplement number
    /// shall be increased. Supplements shall not alter the ordering of
    /// existing CIDs in the character collection. This value shall not
    /// be used in determining compatibility between character collections
    supplement: i32,
}

impl CidSystemInfo {
    pub fn from_dict<'a>(
        mut dict: Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let registry = dict.expect_string("Registry", resolver)?;
        let ordering = dict.expect_string("Ordering", resolver)?;
        let supplement = dict.expect_integer("Supplement", resolver)?;

        Ok(Self {
            registry,
            ordering,
            supplement,
        })
    }
}

#[derive(Debug)]
pub struct CidFontDictionary<'a> {
    /// The PostScript name of the CIDFont. For Type 0 CIDFonts, this shall be
    /// the value of the CIDFontName entry in the CIDFont program. For Type 2
    /// CIDFonts, it shall be derived the same way as for a simple TrueType font.
    /// In either case, the name may have a subset prefix if appropriate
    base_font: String,

    /// A dictionary containing entries that define the character collection of the
    /// CIDFont
    cid_system_info: CidSystemInfo,

    /// A font descriptor describing the CIDFont’s default metrics other than its
    /// glyph widths
    font_descriptor: FontDescriptor<'a>,

    /// The default width for glyphs in the CIDFont
    ///
    /// Default value: 1000 (defined in user units)
    dw: i32,

    /// A description of the widths for the glyphs in the CIDFont
    ///
    /// NOTE: The array’s elements have a variable format that can specify individual
    ///       widths for consecutive CIDs or one width for a range of CIDs
    ///
    /// Default value: none (the DW value shall be used for all glyphs)
    w: Option<Vec<Object<'a>>>,

    /// An array of two numbers specifying the default metrics for vertical writing
    ///
    /// Default value: [880 −1000]
    dw2: [f32; 2],

    /// A description of the metrics for vertical writing for the glyphs in the CIDFont
    ///
    /// Default value: none (the DW2 value shall be used for all glyphs)
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
    cid_to_gid_map: CidToGidMap<'a>,
}

impl<'a> CidFontDictionary<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let base_font = dict.expect_name("BaseFont", resolver)?;
        let cid_system_info =
            CidSystemInfo::from_dict(dict.expect_dict("CIDSystemInfo", resolver)?, resolver)?;
        let font_descriptor =
            FontDescriptor::from_dict(dict.expect_dict("FontDescriptor", resolver)?, resolver)?;
        let dw = dict.get_integer("DW", resolver)?.unwrap_or(1000);
        let w = dict.get_arr("W", resolver)?;
        let dw2 = dict
            .get_arr("DW2", resolver)?
            .map(|arr| -> PdfResult<[f32; 2]> {
                assert_len(&arr, 2)?;

                Ok(arr
                    .into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()?
                    .try_into()
                    .unwrap())
            })
            .transpose()?
            .unwrap_or([880.0, -1000.0]);
        let w2 = dict.get_arr("W2", resolver)?;
        let cid_to_gid_map = dict
            .get_object("CIDToGIDMap", resolver)?
            .map(|obj| CidToGidMap::from_obj(obj, resolver))
            .transpose()?
            .unwrap_or(CidToGidMap::Identity);

        Ok(Self {
            base_font,
            cid_system_info,
            font_descriptor,
            dw,
            w,
            dw2,
            w2,
            cid_to_gid_map,
        })
    }
}

#[derive(Debug)]
enum CidToGidMap<'a> {
    Identity,
    Stream(Stream<'a>),
}

impl<'a> CidToGidMap<'a> {
    pub fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match resolver.resolve(obj)? {
            Object::Name(ref name) if name == "Identity" => Self::Identity,
            Object::Stream(stream) => Self::Stream(stream),
            found => {
                return Err(crate::error::ParseError::MismatchedObjectTypeAny {
                    // found,
                    expected: &[ObjectType::Name, ObjectType::Stream],
                });
            }
        })
    }
}
