use crate::{
    assert_empty,
    error::{ParseError, PdfResult},
    function::{Function, TransferFunction},
    halftones::Halftones,
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    Lexer,
};

#[derive(Debug)]
pub struct GraphicsStateParameters {
    line_width: Option<f32>,
    line_cap_style: Option<LineCapStyle>,
    line_join_style: Option<LineJoinStyle>,
    miter_limit: Option<f32>,
    line_dash_pattern: Option<LineDashPattern>,
    rendering_intent: Option<RenderingIntent>,
    should_overprint_stroking: Option<bool>,
    should_overprint: Option<bool>,
    overprint_mode: Option<i32>,
    // todo
    font: Option<Object>,
    black_generation: Option<Function>,
    // todo: either function or name
    black_generation_two: Option<Function>,
    undercolor_removal: Option<Function>,
    // todo: function or name
    undercolor_removal_two: Option<Function>,
    // todo: function, array, or name
    transfer: Option<Function>,
    transfer_two: Option<Function>,
    halftones: Option<Halftones>,
    flatness_tolerance: Option<f32>,
    smoothness_tolerance: Option<f32>,
    should_apply_automatic_stoke: Option<bool>,
    blend_mode: Option<BlendMode>,

    /// The current soft mask, specifying the mask shape or mask opacity values that shall
    /// be used in the transparent imaging model.
    ///
    /// Although the current soft mask is sometimes referred to as a “soft clip,” altering
    /// it with the gs operator completely replaces the old value with the new one, rather
    /// than intersecting the two as is done with the current clipping path parameter.
    // todo: can also be name
    soft_mask: Option<SoftMask>,
    current_stroking_alpha_constant: Option<f32>,
    current_nonstroking_alpha_constant: Option<f32>,
    alpha_is_shape: Option<bool>,
    is_knockout: Option<bool>,
}

#[derive(Debug)]
pub enum BlendMode {
    Normal,

    /// Same as Normal. This mode exists only for compatibility and should not be used.
    Compatible,

    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,

    Unknown(String),

    /// The application shall use the first blend mode in the array
    /// that it recognizes (or Normal if it recognizes none of them).
    Array(Vec<Self>),
}

impl BlendMode {
    fn from_str(s: String) -> Self {
        match s.as_str() {
            "Normal" => Self::Normal,
            "Compatible" => Self::Compatible,
            "Multiply" => Self::Multiply,
            "Screen" => Self::Screen,
            "Overlay" => Self::Overlay,
            "Darken" => Self::Darken,
            "Lighten" => Self::Lighten,
            "ColorDodge" => Self::ColorDodge,
            "ColorBurn" => Self::ColorBurn,
            "HardLight" => Self::HardLight,
            "SoftLight" => Self::SoftLight,
            "Difference" => Self::Difference,
            "Exclusion" => Self::Exclusion,
            _ => Self::Unknown(s),
        }
    }

    pub fn from_obj(obj: Object, lexer: &mut Lexer) -> PdfResult<Self> {
        Ok(match obj {
            Object::Name(name) => Self::from_str(name),
            Object::Array(objs) => Self::Array(
                objs.into_iter()
                    .map(|obj| lexer.assert_name(obj).map(Self::from_str))
                    .collect::<PdfResult<Vec<Self>>>()?,
            ),
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    found,
                    expected: &[ObjectType::Array, ObjectType::Name],
                })
            }
        })
    }
}

#[derive(Debug)]
pub struct SoftMask {
    subtype: SoftMaskSubtype,
    transparency_group: Stream,
    // todo
    backdrop_color: Option<Vec<Object>>,
    /// A function object (see “Functions”) specifying the transfer function to be used
    /// in deriving the mask values. The function shall accept one input, the computed
    /// group alpha or luminosity (depending on the value of the subtype S), and shall
    /// return one output, the resulting mask value. The input shall be in the range 0.0
    /// to 1.0. The computed output shall be in the range 0.0 to 1.0; if it falls outside
    /// this range, it shall be forced to the nearest valid value. The name Identity may
    /// be specified in place of a function object to designate the identity function.
    ///
    /// Default value: Identity
    transfer_function: Option<TransferFunction>,
}

impl SoftMask {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let subtype = SoftMaskSubtype::from_str(&dict.expect_name("S", lexer)?)?;

        let transparency_group = dict.expect_stream("G", lexer)?;
        let backdrop_color = dict.get_arr("BC", lexer)?;
        let transfer_function = dict
            .get_object("TransferFunction", lexer)?
            .map(TransferFunction::from_obj)
            .transpose()?;

        Ok(Self {
            subtype,
            transparency_group,
            backdrop_color,
            transfer_function,
        })
    }
}

#[derive(Debug)]
enum SoftMaskSubtype {
    /// The group’s computed alpha shall be used, disregarding its colour
    Alpha,

    /// The group’s computed colour shall be converted to a single-component luminosity value
    Luminosity,
}

impl SoftMaskSubtype {
    pub fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "Alpha" => Self::Alpha,
            "Luminosity" => Self::Luminosity,
            found => {
                return Err(ParseError::UnrecognizedVariant {
                    found: found.to_owned(),
                    ty: "SoftMaskSubtype",
                })
            }
        })
    }
}

#[derive(Debug)]
struct LineDashPattern {
    dash_array: Vec<i32>,
    dash_phase: i32,
}

impl LineDashPattern {
    pub fn from_arr(mut arr: Vec<Object>, lexer: &mut Lexer) -> PdfResult<Self> {
        if arr.len() != 2 {
            return Err(ParseError::ArrayOfInvalidLength {
                expected: 2,
                found: arr,
            });
        }

        let dash_phase = lexer.assert_integer(arr.pop().unwrap())?;
        let dash_array = lexer
            .assert_arr(arr.pop().unwrap())?
            .into_iter()
            .map(|obj| lexer.assert_integer(obj))
            .collect::<PdfResult<Vec<i32>>>()?;

        Ok(Self {
            dash_array,
            dash_phase,
        })
    }
}

impl GraphicsStateParameters {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        dict.expect_type("ExtGState", lexer, false)?;

        let line_width = dict.get_number("LW", lexer)?;
        let line_cap_style = dict
            .get_integer("LC", lexer)?
            .map(LineCapStyle::from_integer)
            .transpose()?;
        let line_join_style = dict
            .get_integer("LJ", lexer)?
            .map(LineJoinStyle::from_integer)
            .transpose()?;
        let miter_limit = dict.get_number("ML", lexer)?;
        let line_dash_pattern = dict
            .get_arr("D", lexer)?
            .map(|arr| LineDashPattern::from_arr(arr, lexer))
            .transpose()?;
        let rendering_intent = dict
            .get_name("RI", lexer)?
            .map(|ref s| RenderingIntent::from_str(s))
            .transpose()?;
        let should_overprint_stroking = dict.get_bool("OP", lexer)?;
        let should_overprint = dict.get_bool("op", lexer)?;
        let overprint_mode = dict.get_integer("OPM", lexer)?;
        let font = dict.get_object("Font", lexer)?;
        let black_generation = dict
            .get_object("BG", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let black_generation_two = dict
            .get_object("BG2", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let undercolor_removal = dict
            .get_object("UCR", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let undercolor_removal_two = dict
            .get_object("UCR2", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let transfer = dict
            .get_object("TR", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let transfer_two = dict
            .get_object("TR2", lexer)?
            .map(Function::from_obj)
            .transpose()?;
        let halftones = dict
            .get_object("HT", lexer)?
            .map(|obj| Halftones::from_obj(obj, lexer))
            .transpose()?;

        let flatness_tolerance = dict.get_number("FL", lexer)?;
        let smoothness_tolerance = dict.get_number("SM", lexer)?;
        let should_apply_automatic_stoke = dict.get_bool("SA", lexer)?;

        let blend_mode = dict
            .get_object("BM", lexer)?
            .map(|obj| BlendMode::from_obj(obj, lexer))
            .transpose()?;

        let soft_mask = dict
            .get_dict("SM", lexer)?
            .map(|obj| SoftMask::from_dict(obj, lexer))
            .transpose()?;

        let current_stroking_alpha_constant = dict.get_number("CA", lexer)?;
        let current_nonstroking_alpha_constant = dict.get_number("ca", lexer)?;
        let alpha_is_shape = dict.get_bool("AIS", lexer)?;
        let is_knockout = dict.get_bool("TK", lexer)?;

        assert_empty(dict);

        Ok(GraphicsStateParameters {
            line_width,
            line_cap_style,
            line_join_style,
            miter_limit,
            line_dash_pattern,
            rendering_intent,
            should_overprint_stroking,
            should_overprint,
            overprint_mode,
            font,
            black_generation,
            black_generation_two,
            undercolor_removal,
            undercolor_removal_two,
            transfer,
            transfer_two,
            halftones,
            flatness_tolerance,
            smoothness_tolerance,
            should_apply_automatic_stoke,
            blend_mode,
            soft_mask,
            current_stroking_alpha_constant,
            current_nonstroking_alpha_constant,
            alpha_is_shape,
            is_knockout,
        })
    }
}

#[derive(Debug)]
enum RenderingIntent {
    AbsoluteColorimetric,
    RelativeColorimetric,
    Saturation,
    Perceptual,
}

impl RenderingIntent {
    pub(crate) fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "AbsoluteColorimetric" => Self::AbsoluteColorimetric,
            "RelativeColorimetric" => Self::RelativeColorimetric,
            "Saturation" => Self::Saturation,
            "Perceptual" => Self::Perceptual,
            _ => {
                return Err(ParseError::UnrecognizedVariant {
                    found: s.to_owned(),
                    ty: "RenderingIntent",
                })
            }
        })
    }
}

#[derive(Debug)]
enum LineJoinStyle {
    Miter,
    Round,
    Bevel,
}

impl LineJoinStyle {
    pub fn from_integer(i: i32) -> PdfResult<Self> {
        match i {
            0 => Ok(Self::Miter),
            1 => Ok(Self::Round),
            2 => Ok(Self::Bevel),
            found => Err(ParseError::UnrecognizedVariant {
                found: found.to_string(),
                ty: stringify!(LineJoinStyle),
            }),
        }
    }
}

#[derive(Debug)]
enum LineCapStyle {
    Butt,
    Round,
    ProjectingSquare,
}

impl LineCapStyle {
    pub fn from_integer(i: i32) -> PdfResult<Self> {
        match i {
            0 => Ok(Self::Butt),
            1 => Ok(Self::Round),
            2 => Ok(Self::ProjectingSquare),
            found => Err(ParseError::UnrecognizedVariant {
                found: found.to_string(),
                ty: stringify!(LineCapStyle),
            }),
        }
    }
}
