use crate::{
    catalog::MetadataStream, error::PdfResult, objects::Dictionary, stream::Stream,
    Resolve,
};

#[derive(Debug, Clone)]
pub(crate) struct EmbeddedFontDictionary<'a> {
    /// The length in bytes of the clear-text portion of the Type 1 font program, or the entire
    /// TrueType font program, after it has been decoded using the filters specified by the stream’s
    /// Filter entry, if any
    length_one: Option<u32>,

    /// The length in bytes of the encrypted portion of the Type 1 font program after it has been
    /// decoded using the filters specified by the stream’s Filter entry
    length_two: Option<u32>,

    /// The length in bytes of the fixed-content portion of the Type 1 font program after it has
    /// been decoded using the filters specified by the stream’s Filter entry. If Length3 is 0, it
    /// indicates that the 512 zeros and cleartomark have not been included in the FontFile font
    /// program and shall be added by the conforming reader
    length_three: Option<u32>,

    /// A metadata stream containing metadata for the embedded font program
    metadata: Option<MetadataStream<'a>>,
}

impl<'a> EmbeddedFontDictionary<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let length_one = dict.get_unsigned_integer("Length1", resolver)?;
        let length_two = dict.get_unsigned_integer("Length2", resolver)?;
        let length_three = dict.get_unsigned_integer("Length3", resolver)?;
        let metadata = dict
            .get_stream("Metadata", resolver)?
            .map(|stream| MetadataStream::from_stream(stream, resolver))
            .transpose()?;

        Ok(Self {
            length_one,
            length_two,
            length_three,
            metadata,
        })
    }
}

/// Type 1 font program, in the original (noncompact) format described in Adobe Type 1
/// Font Format. This entry may appear in the font descriptor for a Type1 or MMType1 font
/// dictionary
#[derive(Debug, Clone)]
pub struct Type1FontFile<'a> {
    dict: EmbeddedFontDictionary<'a>,
    pub stream: Stream<'a>,
}

impl<'a> Type1FontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = EmbeddedFontDictionary::from_dict(&mut stream.dict.other, resolver)?;

        Ok(Self { dict, stream })
    }
}

/// TrueType font program, as described in the TrueType Reference Manual. This entry may appear in
/// the font descriptor for a TrueType font dictionary or for a CIDFontType2 CIDFont dictionary
#[derive(Debug, Clone)]
pub struct TrueTypeFontFile<'a> {
    pub(crate) dict: EmbeddedFontDictionary<'a>,
    pub(crate) stream: Stream<'a>,
}

impl<'a> TrueTypeFontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = EmbeddedFontDictionary::from_dict(&mut stream.dict.other, resolver)?;

        Ok(Self { dict, stream })
    }
}

#[derive(Debug)]
pub enum Type3FontFile<'a> {
    CompactType1(CompactType1FontFile<'a>),
    CompactType0Cid(CompactType0CidFontFile<'a>),
    OpenType(OpenTypeFontFile<'a>),
}

/// Type 1–equivalent font program represented in the Compact Font Format (CFF), as described
/// in Adobe Technical Note #5176, The Compact Font Format Specification. This entry may appear
/// in the font descriptor for a Type1 or MMType1 font dictionary
#[derive(Debug)]
pub struct CompactType1FontFile<'a> {
    dict: EmbeddedFontDictionary<'a>,
    stream: Stream<'a>,
}

/// Type 0 CIDFont program represented in the Compact Font Format (CFF), as described in Adobe
/// Technical Note #5176, The Compact Font Format Specification. This entry may appear in the
/// font descriptor for a CIDFontType0 CIDFont dictionary
#[derive(Debug)]
pub struct CompactType0CidFontFile<'a> {
    dict: EmbeddedFontDictionary<'a>,
    stream: Stream<'a>,
}

/// OpenType® font program, as described in the OpenType Specification v.1.4. OpenType is an
/// extension of TrueType that allows inclusion of font programs that use the Compact Font Format
/// (CFF).
///
/// A FontFile3 entry with an OpenType subtype may appear in the font descriptor for these
/// types of font dictionaries:
///   * A TrueType font dictionary or a CIDFontType2 CIDFont dictionary, if the embedded font program
///      contains a “glyf” table. In addition to the “glyf” table, the font program must include these
///      tables: “head”, “hhea”, “hmtx”, “loca”, and “maxp”. The “cvt ” (notice the trailing SPACE), “fpgm”,
///      and “prep” tables must also be included if they are required by the font instructions.
///   * A CIDFontType0 CIDFont dictionary, if the embedded font program contains a “CFF ” table (notice
///      the trailing SPACE) with a Top DICT that uses CIDFont operators (this is equivalent to subtype
///      CIDFontType0C). In addition to the “CFF ” table, the font program must include the “cmap” table.
///   * A Type1 font dictionary or CIDFontType0 CIDFont dictionary, if the embedded font program contains
///      a “CFF ” table without CIDFont operators. In addition to the “CFF ” table, the font program must
///      include the “cmap” table. The OpenType Specification describes a set of required tables; however,
///      not all tables are required in the font file, as described for each type of font dictionary that
///      can include this entry.
///
/// NOTE: The absence of some optional tables (such as those used for advanced line layout) may prevent
///       editing of text containing the font
#[derive(Debug)]
pub struct OpenTypeFontFile<'a> {
    dict: EmbeddedFontDictionary<'a>,
    stream: Stream<'a>,
}

#[pdf_enum]
enum Type3Subtype {
    Type1C = "Type1C",
    CIDFontType0C = "CIDFontType0C",
    OpenType = "OpenType",
}

impl<'a> Type3FontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let subtype = Type3Subtype::from_str(&stream.dict.other.expect_name("Subtype", resolver)?)?;

        Ok(match subtype {
            Type3Subtype::Type1C => {
                Self::CompactType1(CompactType1FontFile::from_stream(stream, resolver)?)
            }
            Type3Subtype::CIDFontType0C => {
                Self::CompactType0Cid(CompactType0CidFontFile::from_stream(stream, resolver)?)
            }
            Type3Subtype::OpenType => {
                Self::OpenType(OpenTypeFontFile::from_stream(stream, resolver)?)
            }
        })
    }
}

impl<'a> CompactType1FontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = EmbeddedFontDictionary::from_dict(&mut stream.dict.other, resolver)?;

        Ok(Self { dict, stream })
    }
}

impl<'a> CompactType0CidFontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = EmbeddedFontDictionary::from_dict(&mut stream.dict.other, resolver)?;

        Ok(Self { dict, stream })
    }
}

impl<'a> OpenTypeFontFile<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = EmbeddedFontDictionary::from_dict(&mut stream.dict.other, resolver)?;

        Ok(Self { dict, stream })
    }
}
