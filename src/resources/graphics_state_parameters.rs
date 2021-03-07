use crate::{
    assert_empty,
    catalog::assert_len,
    error::{ParseError, PdfResult},
    function::{Function, TransferFunction},
    halftones::Halftones,
    objects::{Dictionary, Object, ObjectType},
    pdf_enum,
    stream::Stream,
    Resolve,
};

#[derive(Debug)]
enum FunctionOrDefault {
    Function(Function),
    Default,
}

impl FunctionOrDefault {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        if let Ok(name) = resolver.assert_name(obj.clone()) {
            if name == "Default" {
                return Ok(Self::Default);
            }
        }

        Ok(Self::Function(Function::from_obj(obj, resolver)?))
    }
}

#[derive(Debug)]
pub struct GraphicsStateParameters {
    line_width: Option<f32>,
    line_cap_style: Option<LineCapStyle>,
    line_join_style: Option<LineJoinStyle>,
    miter_limit: Option<f32>,

    /// The line dash pattern, expressed as an array of the form [dashArray dashPhase],
    /// where dashArray shall be itself an array and dashPhase shall be an integer
    line_dash_pattern: Option<LineDashPattern>,

    /// The name of the rendering intent
    rendering_intent: Option<RenderingIntent>,

    /// A flag specifying whether to apply overprint. In PDF 1.2 and earlier, there is a
    /// single overprint parameter that applies to all painting operations. Beginning with PDF
    /// 1.3, there shall be two separate overprint parameters: one for stroking and one for
    /// all other painting operations. Specifying an OP entry shall set both parameters unless
    /// there is also an op entry in the same graphics state parameter dictionary, in which
    /// case the OP entry shall set only the overprint parameter for stroking.
    should_overprint_stroking: Option<bool>,

    /// A flag specifying whether to apply overprint for painting operations other than stroking.
    ///
    /// If this entry is absent, the OP entry, if any, shall also set this parameter.
    should_overprint: Option<bool>,

    overprint_mode: Option<i32>,
    /// An array of the form [font size], where font shall be an indirect reference to a font
    /// dictionary and size shall be a number expressed in text space units. These two objects
    /// correspond to the operands of the Tf operator; however, the first operand shall be an
    /// indirect object reference instead of a resource name.
    // todo
    font: Option<Object>,

    /// The black-generation function, which maps the interval [0.0 1.0] to the interval [0.0 1.0]
    black_generation: Option<Function>,

    /// Same as BG except that the value may also be the name Default, denoting the black-generation
    /// function that was in effect at the start of the page. If both BG and BG2 are present in
    /// the same graphics state parameter dictionary, BG2 shall take precedence
    black_generation_two: Option<FunctionOrDefault>,

    /// The undercolor-removal function, which maps the interval [0.0 1.0] to the interval [-1.0 1.0]
    undercolor_removal: Option<Function>,

    /// Same as UCR except that the value may also be the name Default, denoting the undercolor-removal
    /// function that was in effect at the start of the page. If both UCR and UCR2 are present in the
    /// same graphics state parameter dictionary, UCR2 shall take precedence
    undercolor_removal_two: Option<FunctionOrDefault>,

    /// The transfer function, which maps the interval [0.0 1.0] to the interval [0.0 1.0]. The value
    /// shall be either a single function (which applies to all process colorants) or an array of four
    /// functions (which apply to the process colorants individually). The name Identity may be used to
    /// represent the identity function.
    transfer: Option<TransferFunction>,

    /// Same as TR except that the value may also be the name Default, denoting the transfer function
    /// that was in effect at the start of the page. If both TR and TR2 are present in the same graphics
    /// state parameter dictionary, TR2 shall take precedence
    transfer_two: Option<TransferFunction>,

    /// The halftone dictionary or stream or the name Default, denoting the halftone that was in effect
    /// at the start of the page.
    halftones: Option<Halftones>,

    /// The flatness tolerance controls the maximum permitted distance in device pixels between the
    /// mathematically correct path and an approximation constructed from straight line segments
    flatness_tolerance: Option<f32>,

    /// The smoothness tolerance controls the quality of smooth shading (type 2 patterns and the sh
    /// operator) and thus indirectly controls the rendering performance
    smoothness_tolerance: Option<f32>,

    /// A flag specifying whether to apply automatic stroke adjustment
    should_apply_automatic_stoke: Option<bool>,

    /// The current blend mode to be used in the transparent imaging model
    blend_mode: Option<BlendMode>,

    /// The current soft mask, specifying the mask shape or mask opacity values that shall
    /// be used in the transparent imaging model.
    ///
    /// Although the current soft mask is sometimes referred to as a "soft clip," altering
    /// it with the gs operator completely replaces the old value with the new one, rather
    /// than intersecting the two as is done with the current clipping path parameter.
    // todo: can also be name
    soft_mask: Option<SoftMask>,

    /// The current stroking alpha constant, specifying the constant shape or constant
    /// opacity value that shall be used for stroking operations in the transparent imaging
    /// model
    current_stroking_alpha_constant: Option<f32>,

    /// Same as CA, but for nonstroking operations
    current_nonstroking_alpha_constant: Option<f32>,

    /// The alpha source flag, specifying whether the current soft mask and alpha constant
    /// shall be interpreted as shape values (true) or opacity values (false).
    alpha_is_shape: Option<bool>,

    /// The text knockout flag, shall determine the behaviour of overlapping glyphs within
    /// a text object in the transparent imaging model
    is_knockout: Option<bool>,

    /// Apple-specific rendering hint, whether or not to disable anti-aliasing
    /// Key of "AAPL:AA"
    /// See <http://www.sibelius.com/cgi-bin/helpcenter/chat/chat.pl?com=thread&start=393193&groupid=3&&guest=1>
    apple_antialiasing: Option<bool>,
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

    pub fn from_obj(obj: Object, lexer: &mut dyn Resolve) -> PdfResult<Self> {
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
pub enum SoftMask {
    Dictionary(SoftMaskDictionary),
    None,
}

impl SoftMask {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        if obj.name_is("None") {
            return Ok(Self::None);
        }

        Ok(Self::Dictionary(SoftMaskDictionary::from_dict(
            resolver.assert_dict(obj)?,
            resolver,
        )?))
    }
}

#[derive(Debug)]
pub struct SoftMaskDictionary {
    /// A subtype specifying the method to be used in deriving the mask values from the
    /// transparency group specified by the G entry
    subtype: SoftMaskSubtype,

    /// A transparency group XObject to be used as the source of alpha or colour values
    /// for deriving the mask. If the subtype S is Luminosity, the group attributes
    /// dictionary shall contain a CS entry defining the colour space in which the compositing
    /// computation is to be performed
    transparency_group: Stream,

    /// An array of component values specifying the colour to be used as the backdrop against
    /// which to composite the transparency group XObject G. This entry shall be consulted only
    /// if the subtype S is Luminosity. The array shall consist of n numbers, where n is the
    /// number of components in the colour space specified by the CS entry in the group attributes
    /// dictionary.
    ///
    /// Default value: the colour space's initial value, representing black.
    // todo
    backdrop_color: Option<Vec<Object>>,

    /// A function object specifying the transfer function to be used
    /// in deriving the mask values. The function shall accept one input, the computed
    /// group alpha or luminosity (depending on the value of the subtype S), and shall
    /// return one output, the resulting mask value. The input shall be in the range 0.0
    /// to 1.0. The computed output shall be in the range 0.0 to 1.0; if it falls outside
    /// this range, it shall be forced to the nearest valid value. The name Identity may
    /// be specified in place of a function object to designate the identity function.
    ///
    /// Default value: Identity
    transfer_function: TransferFunction,
}

impl SoftMaskDictionary {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let subtype = SoftMaskSubtype::from_str(&dict.expect_name("S", resolver)?)?;

        let transparency_group = dict.expect_stream("G", resolver)?;
        let backdrop_color = dict.get_arr("BC", resolver)?;
        let transfer_function = dict
            .get_object("TransferFunction", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
            .transpose()?
            .unwrap_or(TransferFunction::Identity);

        Ok(Self {
            subtype,
            transparency_group,
            backdrop_color,
            transfer_function,
        })
    }
}

pdf_enum!(
    #[derive(Debug)]
    enum SoftMaskSubtype {
        /// The group's computed alpha shall be used, disregarding its colour
        Alpha = "Alpha",

        /// The group's computed colour shall be converted to a single-component luminosity value
        Luminosity = "Luminosity",
    }
);

#[derive(Debug)]
pub struct LineDashPattern {
    dash_array: Vec<i32>,
    dash_phase: i32,
}

impl LineDashPattern {
    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 2)?;

        let dash_phase = resolver.assert_integer(arr.pop().unwrap())?;
        let dash_array = resolver
            .assert_arr(arr.pop().unwrap())?
            .into_iter()
            .map(|obj| resolver.assert_integer(obj))
            .collect::<PdfResult<Vec<i32>>>()?;

        Ok(Self {
            dash_array,
            dash_phase,
        })
    }
}

impl GraphicsStateParameters {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type("ExtGState", resolver, false)?;

        let line_width = dict.get_number("LW", resolver)?;
        let line_cap_style = dict
            .get_integer("LC", resolver)?
            .map(LineCapStyle::from_integer)
            .transpose()?;
        let line_join_style = dict
            .get_integer("LJ", resolver)?
            .map(LineJoinStyle::from_integer)
            .transpose()?;
        let miter_limit = dict.get_number("ML", resolver)?;
        let line_dash_pattern = dict
            .get_arr("D", resolver)?
            .map(|arr| LineDashPattern::from_arr(arr, resolver))
            .transpose()?;
        let rendering_intent = dict
            .get_name("RI", resolver)?
            .map(|ref s| RenderingIntent::from_str(s))
            .transpose()?;
        let should_overprint_stroking = dict.get_bool("OP", resolver)?;
        let should_overprint = dict.get_bool("op", resolver)?;
        let overprint_mode = dict.get_integer("OPM", resolver)?;
        let font = dict.get_object("Font", resolver)?;
        let black_generation = dict.get_function("BG", resolver)?;
        let black_generation_two = dict
            .get_object("BG2", resolver)?
            .map(|obj| FunctionOrDefault::from_obj(obj, resolver))
            .transpose()?;
        let undercolor_removal = dict.get_function("UCR", resolver)?;
        let undercolor_removal_two = dict
            .get_object("UCR2", resolver)?
            .map(|obj| FunctionOrDefault::from_obj(obj, resolver))
            .transpose()?;
        let transfer = dict
            .get_object("TR", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
            .transpose()?;
        let transfer_two = dict
            .get_object("TR2", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
            .transpose()?;
        let halftones = dict
            .get_object("HT", resolver)?
            .map(|obj| Halftones::from_obj(obj, resolver))
            .transpose()?;

        let flatness_tolerance = dict.get_number("FL", resolver)?;
        let smoothness_tolerance = dict.get_number("SM", resolver)?;
        let should_apply_automatic_stoke = dict.get_bool("SA", resolver)?;

        let blend_mode = dict
            .get_object("BM", resolver)?
            .map(|obj| BlendMode::from_obj(obj, resolver))
            .transpose()?;

        let soft_mask = dict
            .get_object("SMask", resolver)?
            .map(|obj| SoftMask::from_obj(obj, resolver))
            .transpose()?;

        let current_stroking_alpha_constant = dict.get_number("CA", resolver)?;
        let current_nonstroking_alpha_constant = dict.get_number("ca", resolver)?;
        let alpha_is_shape = dict.get_bool("AIS", resolver)?;
        let is_knockout = dict.get_bool("TK", resolver)?;
        let apple_antialiasing = dict.get_bool("AAPL:AA", resolver)?;

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
            apple_antialiasing,
        })
    }
}

pdf_enum!(
    #[derive(Debug)]
    pub enum RenderingIntent {
        AbsoluteColorimetric = "AbsoluteColorimetric",
        RelativeColorimetric = "RelativeColorimetric",
        Saturation = "Saturation",
        Perceptual = "Perceptual",
    }
);

pdf_enum!(
    int
    #[derive(Debug)]
    enum LineJoinStyle {
        Miter = 0,
        Round = 1,
        Bevel = 2,
    }
);

pdf_enum!(
    int
    #[derive(Debug)]
    enum LineCapStyle {
        Butt = 0,
        Round = 1,
        ProjectingSquare = 2,
    }
);
