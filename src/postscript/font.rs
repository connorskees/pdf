use std::borrow::Cow;

use crate::{data_structures::Matrix, postscript::object::ArrayIndex};

use super::{
    charstring::{CharString, CharStrings},
    interpreter::PostscriptInterpreter,
    object::{PostScriptArray, PostScriptDictionary, PostScriptObject, PostScriptString},
    PostScriptError, PostScriptResult,
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
            _ => anyhow::bail!(PostScriptError::RangeCheck),
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
            _ => anyhow::bail!(PostScriptError::RangeCheck),
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
            _ => anyhow::bail!(PostScriptError::RangeCheck),
        })
    }
}

#[derive(Debug)]
pub(crate) struct Type1PostscriptFont {
    font_info: FontInfo,
    font_name: PostScriptString,
    pub(super) encoding: Encoding,
    paint_type: PaintType,
    font_type: FontType,
    pub font_matrix: Matrix,
    // todo: font_bounding_box: BoundingBox,
    font_bounding_box: ArrayIndex,
    unique_id: Option<i32>,
    metrics: Option<Metrics>,
    stroke_width: Option<f32>,
    pub(super) private: Private,
    pub char_strings: CharStrings,
    // todo: fid: Option<FontId>,
    fid: Option<PostScriptObject>,
}

#[derive(Debug)]
pub(super) struct Encoding {
    codepoint_map: Vec<Option<PostScriptString>>,
}

impl Encoding {
    pub fn from_array(
        arr: &[PostScriptObject],
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let codepoint_map = arr
            .iter()
            .map(|obj| match obj {
                PostScriptObject::Null => Ok(None),
                PostScriptObject::Name(name) => Ok(Some(name.clone())),
                &PostScriptObject::String(s) => Ok(Some(interpreter.get_str(s).clone())),
                _ => anyhow::bail!(PostScriptError::TypeCheck),
            })
            .collect::<PostScriptResult<Vec<Option<PostScriptString>>>>()?;

        Ok(Self { codepoint_map })
    }

    pub fn new(codepoint_map: Vec<Option<PostScriptString>>) -> Self {
        Self { codepoint_map }
    }

    fn not_def() -> PostScriptString {
        PostScriptString::from_bytes(b".notdef".to_vec())
    }

    pub fn get(&self, codepoint: u32) -> Cow<PostScriptString> {
        match self.codepoint_map.get(codepoint as usize) {
            Some(Some(s)) => Cow::Borrowed(s),
            Some(None) | None => Cow::Owned(Self::not_def()),
        }
    }
}

impl Type1PostscriptFont {
    pub(super) fn from_dict(
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

        let encoding = Encoding::from_array(encoding.as_inner(), interpreter)?;

        let paint_type = PaintType::from_integer(
            dict.expect_integer(b"PaintType", PostScriptError::InvalidFont)?,
        )?;
        let font_type = FontType::from_integer(
            dict.expect_integer(b"FontType", PostScriptError::InvalidFont)?,
        )?;

        let font_matrix = interpreter
            .get_arr(dict.expect_array(b"FontMatrix", PostScriptError::InvalidFont)?)
            .as_matrix()?;
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
pub(super) struct Private {
    /// Charstring subroutines
    ///
    /// Required if OtherSubrs are used
    pub(super) subroutines: Option<Vec<CharString>>,

    /// Flex, hint replacement, and future extensions
    ///
    /// Required if Flex or hint replacement are used
    pub(super) other_subroutines: Option<Vec<PostScriptArray>>,

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
    min_feature: ArrayIndex,

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
    pub(super) fn from_dict(
        dict: PostScriptDictionary,
        interpreter: &mut PostscriptInterpreter,
    ) -> PostScriptResult<Self> {
        let subroutines = dict
            .get_array(b"Subrs")?
            .map(|a| interpreter.get_arr(a).clone())
            .map(|arr| {
                arr.into_inner()
                    .into_iter()
                    .filter(|obj| !obj.is_null())
                    .map(|obj| match obj {
                        PostScriptObject::String(s) => {
                            let mut s = interpreter.get_str(s).clone().into_bytes();
                            CharString::parse(&mut s, interpreter.in_pfb)
                        }
                        _ => anyhow::bail!(PostScriptError::TypeCheck),
                    })
                    .collect::<PostScriptResult<Vec<CharString>>>()
            })
            .transpose()?;

        let other_subroutines = dict
            .get_array(b"OtherSubrs")?
            .map(|a| interpreter.get_arr(a).clone())
            .map(|arr| {
                arr.into_inner()
                    .into_iter()
                    .map(|obj: PostScriptObject| match obj {
                        PostScriptObject::Array(p) => Ok(interpreter.get_arr(p).clone()),
                        _ => anyhow::bail!(PostScriptError::TypeCheck),
                    })
                    .collect::<PostScriptResult<Vec<PostScriptArray>>>()
            })
            .transpose()?;

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
struct BlueValues;
