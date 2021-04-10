use std::{collections::HashMap, convert::TryInto};

use crate::{
    data_structures::{Matrix, Rectangle},
    pdf_enum,
};

use super::{
    decode::{decrypt, decrypt_charstring},
    object::{PostScriptArray, PostScriptDictionary, PostScriptObject, PostScriptString},
    GraphicsOperator, PostScriptError, PostScriptResult, PostscriptInterpreter,
};

pdf_enum!(
    int
    #[derive(Debug, Clone, Copy)]
    enum FontType {
        One = 1,
        Three = 3,
    }
);

pdf_enum!(
    int
    #[derive(Debug, Clone, Copy)]
    enum LanguageGroup {
        /// Language group 0 consists of languages that use Latin, Greek,
        /// Cyrillic, and similar alphabets
        NonCjk = 0,

        /// Language group 1 consists of Chinese ideographs and similar
        /// character sets, including Japanese Kanji and Korean Hangul
        Cjk = 1,
    }
);

#[derive(Debug)]
pub(super) struct Type1PostscriptFont {
    font_info: FontInfo,
    font_name: PostScriptString,
    encoding: PostScriptArray,
    paint_type: i32,
    font_type: FontType,
    font_matrix: Matrix,
    font_bounding_box: Rectangle,
    unique_id: i32,
    metrics: Metrics,
    stroke_width: f32,
    private: Private,
    char_strings: CharStrings,
    fid: Option<FontId>,
}

impl Type1PostscriptFont {
    pub fn from_dict(
        mut dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let font_info = FontInfo::from_dict(
            interpreter
                .get_dict(&dict.expect_dict(b"FontInfo", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        let char_strings = CharStrings::from_dict(
            interpreter
                .get_dict(&dict.expect_dict(b"CharStrings", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        dbg!(char_strings);

        let private = Private::from_dict(
            interpreter
                .get_dict(&dict.expect_dict(b"Private", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        dbg!(private);

        todo!()
    }
}

#[derive(Debug)]
struct FontId;

#[derive(Debug)]
struct FontInfo {
    /// Version number of the font program
    version: Option<PostScriptString>,

    /// Trademark or copyright notice, if applicable
    notice: Option<PostScriptString>,

    /// Unique, human-readable name for an individual font
    full_name: Option<PostScriptString>,

    /// Human-readable name for a group of fonts that are
    /// stylistic variants of a single design. All fonts
    /// that are members of such a group should have exactly
    /// the same FamilyName
    family_name: Option<PostScriptString>,

    /// Human-readable name for the weight, or “boldness,”
    /// attribute of a font
    weight: Option<PostScriptString>,

    /// Angle in degrees counterclockwise from the vertical
    /// of the dominant vertical strokes of the font
    italic_angle: Option<f32>,

    ///  If true, indicates that the font is a fixed-pitch
    /// (monospaced) font
    is_fixed_pitch: Option<bool>,

    /// Recommended distance from the baseline for positioning
    /// underlining strokes
    ///
    /// This number is the y coordinate (in character space)
    /// of the center of the stroke
    underline_position: Option<f32>,

    /// Recommended stroke width for underlining, in units of
    /// the character coordinate system
    underline_thickness: Option<f32>,
}

impl FontInfo {
    pub fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let version = dict
            .get_str(b"version")?
            .map(|s| interpreter.get_str(&s).clone());
        let full_name = dict
            .get_str(b"FullName")?
            .map(|s| interpreter.get_str(&s).clone());
        let weight = dict
            .get_str(b"Weight")?
            .map(|s| interpreter.get_str(&s).clone());
        let family_name = dict
            .get_str(b"FamilyName")?
            .map(|s| interpreter.get_str(&s).clone());
        let notice = dict
            .get_str(b"Notice")?
            .map(|s| interpreter.get_str(&s).clone());
        let italic_angle = dict.get_number(b"ItalicAngle")?;
        let underline_position = dict.get_number(b"UnderlinePosition")?;
        let underline_thickness = dict.get_number(b"UnderlineThickness")?;
        let is_fixed_pitch = dict.get_bool(b"isFixedPitch")?;

        Ok(Self {
            version,
            full_name,
            weight,
            family_name,
            italic_angle,
            underline_position,
            underline_thickness,
            is_fixed_pitch,
            notice,
        })
    }
}

#[derive(Debug)]
struct Metrics;

#[derive(Debug)]
struct Private {
    subroutines: Vec<Subroutine>,
    other_subroutines: Vec<Subroutine>,
    unique_id: i32,
    blue_values: Vec<i32>,
    other_blues: Vec<i32>,
    family_blues: Vec<i32>,
    family_other_blues: Vec<i32>,
    blue_scale: f32,
    blue_shift: i32,
    blue_fuzz: i32,
    std_hw: Vec<i32>,
    std_vw: Vec<i32>,
    stem_snap_h: Vec<i32>,
    stem_snap_v: Vec<i32>,
    force_bold: bool,
    language_group: LanguageGroup,
    password: i32,

    /// The lenIV entry is an integer specifying the number of random bytes
    /// at the beginning of charstrings for charstring encryption.
    ///
    /// The default value of lenIV is 4.
    ///
    /// To be compatible with version 23.0 of the PostScript interpreter
    /// (found in the original LaserWriter®), the value of lenIV should be
    /// set to 4. If compatibility with version 23.0 printers is not necessary,
    /// lenIV can be set to 0 or 1 to save storage
    len_iv: i32,
    min_feature: Vec<i32>,
    rnd_stem_up: i32,

    /// The optional ExpansionFactor entry is a real number that gives a
    /// limit for changing the size of a character bounding box during the
    /// processing that adjusts the sizes of counters in fonts of LanguageGroup
    /// 1
    ///
    /// The default value of ExpansionFactor is 0.06
    expansion_factor: f32,
}

impl Private {
    pub fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
struct CharStrings(HashMap<PostScriptString, CharString>);

impl CharStrings {
    pub fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let mut char_strings = HashMap::new();

        for (key, value) in dict.into_iter() {
            let char_string = match value {
                PostScriptObject::String(s) => {
                    CharString::parse(interpreter.get_str(&s).clone().as_bytes())?
                }
                _ => return Err(PostScriptError::TypeCheck),
            };

            char_strings.insert(key, char_string);
        }

        Ok(Self(char_strings))
    }
}

#[derive(Debug)]
enum CharStringStackObject {
    Integer(i32),
    Operator(GraphicsOperator),
}

#[derive(Debug)]
struct CharString(Vec<CharStringStackObject>);

#[derive(Debug)]
struct CharStringStack {
    stack: [i32; 24],
    end: u8,
}

impl CharString {
    pub fn parse(b: &[u8]) -> PostScriptResult<Self> {
        // dbg!(b[0] as char);
        let b = decrypt_charstring(b);

        let mut i = 0;

        let mut objs = Vec::new();

        while i < b.len() {
            match b[i] {
                v @ 0..=31 => match v {
                    1 => todo!("hstem"),
                    3 => todo!("vstem"),
                    4 => todo!("vmoveto"),
                    5 => todo!("rlineto"),
                    6 => todo!("hlineto"),
                    7 => todo!("vlineto"),
                    8 => todo!("rrcurveto"),
                    9 => todo!("closepath"),
                    10 => todo!("callsubr"),
                    11 => todo!("return"),
                    12 => todo!("escape"),
                    13 => todo!("hsbw"),
                    14 => todo!("endchar"),
                    21 => todo!("rmoveto"),
                    22 => todo!("hmoveto"),
                    30 => todo!("vhcurveto"),
                    31 => todo!("hvcurveto"),
                    _ => todo!("INVALID OP CODE: {}", v),
                },

                // A charstring byte containing a value, v, between 32 and
                // 246 inclusive, indicates the integer v − 139. Thus, the
                // integer values from −107 through 107 inclusive may be
                // encoded in a single byte
                v @ 32..=246 => {
                    i += 1;

                    objs.push(CharStringStackObject::Integer(v as i32 - 139));
                }

                // A charstring byte containing a value, v, between 247 and
                // 250 inclusive, indicates an integer involving the next byte,
                // w, according to the formula:
                //
                //   [(v − 247) × 256] + w + 108
                //
                // Thus, the integer values between 108 and 1131 inclusive
                // can be encoded in 2 bytes in this manner
                v @ 247..=250 => {
                    todo!("{}", v)
                }

                // A charstring byte containing a value, v, between 251 and
                // 254 inclusive, indicates an integer involving the next
                // byte, w, according to the formula:
                //
                // − [(v − 251) × 256] − w − 108
                //
                // Thus, the integer values between −1131 and −108 inclusive
                // can be encoded in 2 bytes in this manner
                v @ 251..=254 => {
                    i += 1;
                    let w = b[i] as i32;
                    let int = -((v as i32 - 251) * 256) - w - 108;

                    objs.push(CharStringStackObject::Integer(int));
                }

                // Finally, if the charstring byte contains the value 255,
                // the next four bytes indicate a two’s complement signed integer.
                // The first of these four bytes contains the highest order
                // bits, the second byte contains the next higher order bits
                // and the fourth byte contains the lowest order bits. Thus,
                // any 32-bit signed integer may be encoded in 5 bytes in this
                // manner (the 255 byte plus 4 more bytes)
                255 => {
                    i += 1;
                    let bytes = &b[i..(i + 4)];

                    i += 5;

                    let int = i32::from_be_bytes(bytes.try_into().unwrap());

                    objs.push(CharStringStackObject::Integer(int));
                }
            }
        }

        Ok(Self(objs))
    }
}

#[derive(Debug)]
struct Subroutine;

// impl Type1PostscriptFont {
//     pub fn new(dict: PostScriptDictionary) -> PostScriptResult<Self> {

//         dbg!(encoding);

//         Ok(Self {})
//     }
// }
