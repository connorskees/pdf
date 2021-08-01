use std::{collections::HashMap, convert::TryInto};

use crate::postscript::object::ArrayIndex;

use super::{
    decode::decrypt_charstring,
    object::{
        PostScriptArray, PostScriptDictionary, PostScriptObject, PostScriptString, Procedure,
    },
    GraphicsOperator, PostScriptError, PostScriptResult, PostscriptInterpreter,
};

#[derive(Debug, Clone, Copy)]
enum FontType {
    One = 1,
    Three = 3,
}

impl FontType {
    pub fn from_integer(i: i32) -> PostScriptResult<Self> {
        Ok(match i {
            1 => FontType::One,
            3 => FontType::Three,
            _ => return Err(PostScriptError::RangeCheck),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum PaintType {
    Fill = 0,
    Outline = 2,
}

impl PaintType {
    pub fn from_integer(i: i32) -> PostScriptResult<Self> {
        Ok(match i {
            0 => PaintType::Fill,
            2 => PaintType::Outline,
            _ => return Err(PostScriptError::RangeCheck),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum LanguageGroup {
    /// Language group 0 consists of languages that use Latin, Greek,
    /// Cyrillic, and similar alphabets
    NonCjk = 0,

    /// Language group 1 consists of Chinese ideographs and similar
    /// character sets, including Japanese Kanji and Korean Hangul
    Cjk = 1,
}

impl LanguageGroup {
    pub fn from_integer(i: i32) -> PostScriptResult<Self> {
        Ok(match i {
            0 => LanguageGroup::NonCjk,
            1 => LanguageGroup::Cjk,
            _ => return Err(PostScriptError::RangeCheck),
        })
    }
}

#[derive(Debug)]
pub(super) struct Type1PostscriptFont {
    font_info: FontInfo,
    font_name: PostScriptString,
    encoding: PostScriptArray,
    paint_type: PaintType,
    font_type: FontType,
    // todo: font_matrix: Matrix,
    font_matrix: PostScriptArray,
    // todo: font_bounding_box: Rectangle,
    font_bounding_box: Procedure,
    unique_id: Option<i32>,
    metrics: Option<Metrics>,
    stroke_width: Option<f32>,
    private: Private,
    char_strings: CharStrings,
    // todo: fid: Option<FontId>,
    fid: Option<PostScriptObject>,
}

impl Type1PostscriptFont {
    pub fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let font_info = FontInfo::from_dict(
            interpreter
                .get_dict(dict.expect_dict(b"FontInfo", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        let font_name = dict.expect_name(b"FontName", PostScriptError::InvalidFont)?;

        let encoding = interpreter
            .get_arr(dict.expect_array(b"Encoding", PostScriptError::InvalidFont)?)
            .clone();

        let paint_type = PaintType::from_integer(
            dict.expect_integer(b"PaintType", PostScriptError::InvalidFont)?,
        )?;
        let font_type = FontType::from_integer(
            dict.expect_integer(b"FontType", PostScriptError::InvalidFont)?,
        )?;

        let font_matrix = interpreter
            .get_arr(dict.expect_array(b"FontMatrix", PostScriptError::InvalidFont)?)
            .clone();
        let font_bounding_box = dict.expect_procedure(b"FontBBox", PostScriptError::InvalidFont)?;
        let unique_id = dict.get_integer(b"UniqueID")?;

        let metrics = dict
            .get_dict(b"Metrics")?
            .map(|idx| Metrics::from_dict(interpreter.get_dict(idx).clone(), interpreter))
            .transpose()?;

        let stroke_width = dict.get_number(b"StrokeWidth")?;

        let private = Private::from_dict(
            interpreter
                .get_dict(dict.expect_dict(b"Private", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        let char_strings = CharStrings::from_dict(
            interpreter
                .get_dict(dict.expect_dict(b"CharStrings", PostScriptError::InvalidFont)?)
                .clone(),
            interpreter,
        )?;

        let font_id = dict
            .get(&PostScriptString::from_bytes(b"fontID".to_vec()))
            .cloned();

        Ok(Self {
            font_info,
            font_name,
            encoding,
            paint_type,
            font_type,
            font_matrix,
            font_bounding_box,
            unique_id,
            metrics,
            stroke_width,
            private,
            char_strings,
            fid: font_id,
        })
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
            .map(|s| interpreter.get_str(s).clone());
        let full_name = dict
            .get_str(b"FullName")?
            .map(|s| interpreter.get_str(s).clone());
        let weight = dict
            .get_str(b"Weight")?
            .map(|s| interpreter.get_str(s).clone());
        let family_name = dict
            .get_str(b"FamilyName")?
            .map(|s| interpreter.get_str(s).clone());
        let notice = dict
            .get_str(b"Notice")?
            .map(|s| interpreter.get_str(s).clone());
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

impl Metrics {
    pub fn from_dict(
        dict: PostScriptDictionary,
        _interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        dbg!(dict);
        todo!()
    }
}

// todo: a lot more depth to the array entries here
#[derive(Debug)]
struct Private {
    /// Charstring subroutines
    ///
    /// Required if OtherSubrs are used
    subroutines: Option<ArrayIndex>,

    /// Flex, hint replacement, and future extensions
    ///
    /// Required if Flex or hint replacement are used
    other_subroutines: Option<ArrayIndex>,

    /// Number unique to each Type 1 font program
    ///
    /// Optional, but strongly recommended
    unique_id: Option<i32>,

    /// Font-wide vertical alignment zones
    blue_values: ArrayIndex,

    /// Additional bottom alignment zones
    other_blues: Option<ArrayIndex>,

    ///  Family-wide vertical alignment zones
    family_blues: Option<ArrayIndex>,

    /// Family-wide bottom alignment zones
    family_other_blues: Option<ArrayIndex>,

    /// Related to point size at which to deactivate overshoot suppression
    blue_scale: Option<f32>,

    ///  Overshoot enforcement. If Flex feature is used, then the maximum Flex
    /// feature height plus 1
    blue_shift: Option<i32>,

    /// Extends the range of alignment zones
    blue_fuzz: Option<i32>,

    /// Dominant horizontal stem width
    std_hw: Option<ArrayIndex>,

    /// Dominant vertical stem width
    std_vw: Option<ArrayIndex>,

    /// Array of common horizontal stem widths
    stem_snap_h: Option<ArrayIndex>,

    /// Array of common vertical stem widths
    stem_snap_v: Option<ArrayIndex>,

    /// Set to true to force bold appearance at small sizes. Set to false to
    /// inhibit this behavior
    force_bold: Option<bool>,

    /// Identifies language group of font
    language_group: Option<LanguageGroup>,

    /// Compatibility entry. Set to 5839
    ///
    /// Required
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
    len_iv: Option<i32>,

    /// Obsolete. Set to {16 16}
    ///
    /// Required
    min_feature: Procedure,

    /// Compatibility entry. Use only for font programs in language group 1
    rnd_stem_up: Option<i32>,

    /// The optional ExpansionFactor entry is a real number that gives a
    /// limit for changing the size of a character bounding box during the
    /// processing that adjusts the sizes of counters in fonts of LanguageGroup 1
    ///
    /// The default value of ExpansionFactor is 0.06
    expansion_factor: f32,
}

impl Private {
    pub fn from_dict(
        dict: PostScriptDictionary,
        _interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let subroutines = dict.get_array(b"Subrs")?;
        let other_subroutines = dict.get_array(b"OtherSubrs")?;
        let unique_id = dict.get_integer(b"UniqueID")?;
        let blue_values = dict.expect_array(b"BlueValues", PostScriptError::InvalidFont)?;
        let other_blues = dict.get_array(b"OtherBlues")?;
        let family_blues = dict.get_array(b"FamilyBlues")?;
        let family_other_blues = dict.get_array(b"FamilyOtherBlues")?;
        let blue_scale = dict.get_number(b"BlueScale")?;
        let blue_shift = dict.get_integer(b"BlueShift")?;
        let blue_fuzz = dict.get_integer(b"BlueFuzz")?;
        let std_hw = dict.get_array(b"StdHW")?;
        let std_vw = dict.get_array(b"StdVW")?;
        let stem_snap_h = dict.get_array(b"StemSnapH")?;
        let stem_snap_v = dict.get_array(b"StemSnapV")?;
        let force_bold = dict.get_bool(b"ForceBold")?;
        let language_group = dict
            .get_integer(b"LanguageGroup")?
            .map(LanguageGroup::from_integer)
            .transpose()?;
        let password = dict.expect_integer(b"password", PostScriptError::InvalidFont)?;
        let len_iv = dict.get_integer(b"lenIV")?;
        let min_feature = dict.expect_procedure(b"MinFeature", PostScriptError::InvalidFont)?;
        let rnd_stem_up = dict.get_integer(b"RndStemUp")?;
        let expansion_factor = dict.get_number(b"ExpansionFactor")?.unwrap_or(0.06);

        Ok(Self {
            subroutines,
            other_subroutines,
            unique_id,
            blue_values,
            other_blues,
            family_blues,
            family_other_blues,
            blue_scale,
            blue_shift,
            blue_fuzz,
            std_hw,
            std_vw,
            stem_snap_h,
            stem_snap_v,
            force_bold,
            language_group,
            password,
            len_iv,
            min_feature,
            rnd_stem_up,
            expansion_factor,
        })
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
                    CharString::parse(interpreter.get_str(s).clone().as_bytes())?
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
        let b = decrypt_charstring(b);

        let mut i = 0;

        let mut objs = Vec::new();

        while i < b.len() {
            let byte = b[i];

            i += 1;

            match byte {
                v @ 0..=31 => match v {
                    1 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::HorizontalStem,
                    )),
                    3 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::VerticalStem,
                    )),
                    4 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::VerticalMoveTo,
                    )),
                    5 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::RelativeLineTo,
                    )),
                    6 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::HorizontalLineTo,
                    )),
                    7 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::VerticalLineTo,
                    )),
                    8 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::RelativeRelativeCurveTo,
                    )),
                    9 => objs.push(CharStringStackObject::Operator(GraphicsOperator::ClosePath)),
                    10 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::CallSubroutine,
                    )),
                    11 => objs.push(CharStringStackObject::Operator(GraphicsOperator::Return)),
                    12 => {
                        match b[i] {
                            0 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::DotSection,
                            )),
                            1 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::VerticalStem3,
                            )),
                            2 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::HorizontalStem3,
                            )),
                            6 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::StandardEncodingAccentedCharacter,
                            )),
                            7 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::SideBearingWidth,
                            )),
                            12 => objs.push(CharStringStackObject::Operator(GraphicsOperator::Div)),
                            16 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::CallOtherSubroutine,
                            )),
                            17 => objs.push(CharStringStackObject::Operator(GraphicsOperator::Pop)),
                            33 => objs.push(CharStringStackObject::Operator(
                                GraphicsOperator::SetCurrentPoint,
                            )),
                            v => todo!("INVALID OP CODE: 12 {}", v),
                        }

                        i += 1;
                    }
                    13 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::HorizontalSideBearingWidth,
                    )),
                    14 => objs.push(CharStringStackObject::Operator(GraphicsOperator::EndChar)),
                    21 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::RelativeMoveTo,
                    )),
                    22 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::HorizontalMoveTo,
                    )),
                    30 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::VerticalHorizontalCurveTo,
                    )),
                    31 => objs.push(CharStringStackObject::Operator(
                        GraphicsOperator::HorizontalVerticalCurveTo,
                    )),
                    v => todo!("INVALID OP CODE: {}", v),
                },

                // A charstring byte containing a value, v, between 32 and
                // 246 inclusive, indicates the integer v − 139. Thus, the
                // integer values from −107 through 107 inclusive may be
                // encoded in a single byte
                v @ 32..=246 => objs.push(CharStringStackObject::Integer(v as i32 - 139)),

                // A charstring byte containing a value, v, between 247 and
                // 250 inclusive, indicates an integer involving the next byte,
                // w, according to the formula:
                //
                //   [(v − 247) × 256] + w + 108
                //
                // Thus, the integer values between 108 and 1131 inclusive
                // can be encoded in 2 bytes in this manner
                v @ 247..=250 => {
                    let w = b[i] as i32;
                    let int = ((v as i32 - 247) * 256) + w + 108;

                    i += 1;

                    objs.push(CharStringStackObject::Integer(int));
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
                    let w = b[i] as i32;
                    let int = -((v as i32 - 251) * 256) - w - 108;

                    i += 1;

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

#[derive(Debug)]
struct BlueValues;
