mod error;
mod instruction;
pub(crate) mod parse;
mod state;
pub(crate) mod table;

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
pub struct TrueTypeFont<'a> {
    pub(crate) base: BaseFontDict<'a>,

    base_font: String,

    encoding: Option<FontEncoding>,
}

impl<'a> TrueTypeFont<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let base = BaseFontDict::from_dict(&mut dict, resolver)?;
        let base_font = dict.expect_name("BaseFont", resolver)?;
        let encoding = dict.get::<FontEncoding>("Encoding", resolver)?;

        Ok(Self {
            base,
            base_font,
            encoding,
        })
    }
}

/// 16-bit signed fraction
struct ShortFraction(i16);

/// 16.16-bit signed fixed-point number
struct Fixed(i32);

enum DataType {
    /// 16-bit signed fraction
    ShortFraction(ShortFraction),

    /// 16.16-bit signed fixed-point number
    Fixed(i32),

    /// 16-bit signed integer that describes a quantity in FUnits, the smallest
    /// measurable distance in em space
    FWord(FWord),

    /// 16-bit unsigned integer that describes a quantity in FUnits, the smallest
    /// measurable distance in em space
    UnsignedFWord(u16),

    /// 16-bit signed fixed number with the low 14 bits representing fraction.
    F2Dot14(i16),

    /// The long internal format of a date in seconds since 12:00 midnight, January
    /// 1, 1904. It is represented as a signed 64-bit integer
    LongDateTime(LongDateTime),
}

pub struct LongDateTime(i64);
pub struct FWord(i16);
