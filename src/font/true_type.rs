use crate::{error::PdfResult, objects::Dictionary, Resolve};

use super::{encoding::FontEncoding, BaseFontDict};

/// A TrueType font dictionary may contain the same entries as a Type 1 font dictionary, with these differences:
///   * The value of Subtype shall be TrueType
///   * The value of Encoding is subject to limitations
///   * The value of BaseFont is derived differently
///
/// The PostScript name for the value of BaseFont may be determined in one of two ways:
///   * If the TrueType font program's “name” table contains a PostScript name, it shall be used.
///   * In the absence of such an entry in the “name” table, a PostScript name shall be derived from the name
///      by which the font is known in the host operating system. On a Windows system, the name shall be based
///      on the lfFaceName field in a LOGFONT structure; in the Mac OS, it shall be based on the name of the FOND
///      resource. If the name contains any SPACEs, the SPACEs shall be removed.
#[derive(Debug)]
pub struct TrueTypeFont {
    base: BaseFontDict,

    base_font: String,

    encoding: Option<FontEncoding>,
}

impl TrueTypeFont {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let base = BaseFontDict::from_dict(&mut dict, resolver)?;
        let base_font = dict.expect_name("BaseFont", resolver)?;
        let encoding = dict
            .get_object("Encoding", resolver)?
            .map(|obj| FontEncoding::from_obj(obj, resolver))
            .transpose()?;

        Ok(Self {
            base,
            base_font,
            encoding,
        })
    }
}
