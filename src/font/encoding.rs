use std::{collections::HashMap, convert::TryFrom, mem};

use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType},
    pdf_enum, Resolve,
};

#[derive(Debug)]
pub enum FontEncoding {
    Base(BaseFontEncoding),
    Dictionary(FontEncodingDict),
}

impl FontEncoding {
    pub fn from_obj<'a>(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match resolver.resolve(obj)? {
            Object::Name(ref name) => Self::Base(BaseFontEncoding::from_str(name)?),
            Object::Dictionary(dict) => {
                Self::Dictionary(FontEncodingDict::from_dict(dict, resolver)?)
            }
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    // found,
                    expected: &[ObjectType::Dictionary, ObjectType::Name],
                });
            }
        })
    }
}

pdf_enum!(
    #[derive(Debug)]
    pub enum BaseFontEncoding {
        /// Mac OS standard encoding for Latin text in Western writing systems.
        ///
        /// Conforming readers shall have a predefined encoding named MacRomanEncoding that may be used with
        /// both Type 1 and TrueType fonts.
        MacRomanEncoding = "MacRomanEncoding",

        /// An encoding for use with expert fonts-ones containing the expert character set.
        ///
        /// Conforming readers shall have a predefined encoding named MacExpertEncoding. Despite its
        /// name, it is not a platform specific encoding; however, only certain fonts have the
        /// appropriate character set for use with this encoding. No such fonts are among the
        /// standard 14 predefined fonts.
        MacExpertEncoding = "MacExpertEncoding",

        /// Windows Code Page 1252, often called the "Windows ANSI" encoding.
        ///
        /// This is the standard Windows encoding for Latin text in Western writing systems. Conforming
        /// readers shall have a predefined encoding named WinAnsiEncoding that may be used with both
        /// Type 1 and TrueType fonts.
        WinAnsiEncoding = "WinAnsiEncoding",
    }
);

#[derive(Debug)]
pub struct FontEncodingDict {
    /// The base encoding—that is, the encoding from which the Differences entry (if present)
    /// describes differences— shall be the name of one of the predefined encodings
    /// MacRomanEncoding, MacExpertEncoding, or WinAnsiEncoding. If this entry is absent,
    /// the Differences entry shall describe differences from an implicit base encoding. For
    /// a font program that is embedded in the PDF file, the implicit base encoding shall be
    /// the font program’s built-in encoding. Otherwise, for a nonsymbolic font, it shall be
    /// StandardEncoding, and for a symbolic font, it shall be the font’s built-in encoding
    base_encoding: Option<BaseFontEncoding>,

    /// An array describing the differences from the encoding specified by BaseEncoding or,
    /// if BaseEncoding is absent, from an implicit base encoding. The Differences array is
    /// described in subsequent sub-clauses.
    differences: FontDifferences,
}

impl FontEncodingDict {
    const TYPE: &'static str = "Encoding";

    pub fn from_dict<'a>(
        mut dict: Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, false)?;

        let base_encoding = dict
            .get_name("BaseEncoding", resolver)?
            .as_deref()
            .map(BaseFontEncoding::from_str)
            .transpose()?;
        let differences =
            FontDifferences::from_arr(dict.expect_arr("Differences", resolver)?, resolver)?;

        Ok(Self {
            base_encoding,
            differences,
        })
    }
}

#[derive(Debug)]
struct FontDifferences(HashMap<u32, Vec<String>>);

impl FontDifferences {
    pub fn from_arr<'a>(
        mut arr: Vec<Object<'a>>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        if arr.is_empty() {
            return Ok(FontDifferences(HashMap::new()));
        }

        let mut map = HashMap::new();

        let mut code_point = resolver.assert_unsigned_integer(arr.remove(0))?;
        let mut names = Vec::new();

        for obj in arr.into_iter().skip(1) {
            match resolver.resolve(obj)? {
                Object::Integer(i) => {
                    map.insert(code_point, mem::take(&mut names));
                    names.clear();
                    code_point = u32::try_from(i)?;
                }
                Object::Name(name) => names.push(name),
                found => {
                    return Err(ParseError::MismatchedObjectTypeAny {
                        // found,
                        expected: &[ObjectType::Name, ObjectType::Integer],
                    });
                }
            }
        }

        Ok(Self(map))
    }
}
