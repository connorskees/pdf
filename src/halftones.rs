use crate::{
    catalog::ColorSpace,
    error::{ParseError, PdfResult},
    function::{SpotFunction, TransferFunction},
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    Lexer,
};

#[derive(Debug)]
pub enum Halftones {
    Default,
    Dictionary(HalftoneDictionary),
    Stream(HalftoneStream),
}

impl Halftones {
    pub fn from_obj(obj: Object, lexer: &mut Lexer) -> PdfResult<Self> {
        Ok(match obj {
            Object::Name(name) if name == "Default" => Halftones::Default,
            Object::Stream(stream) => Halftones::Stream(HalftoneStream::from_stream(stream)?),
            Object::Dictionary(dict) => {
                Halftones::Dictionary(HalftoneDictionary::from_dict(dict, lexer)?)
            }
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Name, ObjectType::Stream, ObjectType::Dictionary],
                    found,
                })
            }
        })
    }
}

#[derive(Debug)]
pub struct HalftoneStream {}

impl HalftoneStream {
    pub fn from_stream(_stream: Stream) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub enum HalftoneDictionary {
    One(HalftoneOne),
    Five(HalftoneFive),
    Six(HalftoneSix),
    Ten(HalftoneTen),
    Sixteen(HalftoneSixteen),
}

enum HalftoneType {
    /// Defines a single halftone screen by a frequency, angle, and spot function
    One = 1,

    /// Defines an arbitrary number of halftone screens, one for each colorant or colour
    /// component (including both primary and spot colorants). The keys in this dictionary
    /// are names of colorants; the values are halftone dictionaries of other types, each
    /// defining the halftone screen for a single colorant.
    Five = 5,

    /// Defines a single halftone screen by a threshold array containing 8-bit sample values.
    Six = 6,

    /// Defines a single halftone screen by a threshold array containing 8-bit sample values,
    /// representing a halftone cell that may have a nonzero screen angle
    Ten = 10,

    /// Defines a single halftone screen by a threshold array containing 16-bit sample values,
    /// representing a halftone cell that may have a nonzero screen angle.
    Sixteen = 16,
}

impl HalftoneType {
    pub fn from_integer(i: i32) -> PdfResult<Self> {
        Ok(match i {
            1 => Self::One,
            5 => Self::Five,
            6 => Self::Six,
            10 => Self::Ten,
            16 => Self::Sixteen,
            _ => {
                return Err(ParseError::UnrecognizedVariant {
                    ty: "HalftoneType",
                    found: i.to_string(),
                })
            }
        })
    }
}

#[derive(Debug)]
pub struct HalftoneOne {
    /// The screen frequency, measured in halftone cells per inch in device space
    frequency: f32,

    /// The screen angle, in degrees of rotation counterclockwise with respect to
    /// the device coordinate system
    angle: f32,

    /// A function object defining the order in which device pixels within a screen
    /// cell shall be adjusted for different gray levels, or the name of one of the
    /// predefined spot functions
    spot_function: SpotFunction,

    /// A flag specifying whether to invoke a special halftone algorithm that is extremely
    /// precise but computationally expensive; see Note 1 for further discussion.
    ///
    /// Default value: false
    accurate_screens: Option<bool>,

    /// A transfer function, which overrides the current transfer function in the graphics
    /// state for the same component.
    ///
    /// This entry shall be present if the dictionary is a component of a type 5 halftone and
    /// represents either a nonprimary or nonstandard primary colour component.
    ///
    /// The name Identity may be used to specify the identity function
    transfer_function: Option<TransferFunction>,
}

impl HalftoneOne {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let frequency = dict.expect_number("Frequency", lexer)?;
        let angle = dict.expect_number("Angle", lexer)?;
        let spot_function = SpotFunction::from_obj(dict.expect_object("SpotFunction", lexer)?)?;
        let accurate_screens = dict.get_bool("AccurateScreens", lexer)?;
        let transfer_function = dict
            .get_object("TransferFunction", lexer)?
            .map(|obj| TransferFunction::from_obj(obj))
            .transpose()?;

        Ok(HalftoneOne {
            frequency,
            angle,
            spot_function,
            accurate_screens,
            transfer_function,
        })
    }
}

#[derive(Debug)]
pub struct HalftoneFive {
    colorant: ColorSpace,
    default: Box<Halftones>,
}

impl HalftoneFive {
    pub fn from_dict(mut _dict: Dictionary, _lexer: &mut Lexer) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct HalftoneSix {
    width: i32,
    height: i32,
    transfer_function: Option<TransferFunction>,
}

impl HalftoneSix {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let width = dict.expect_integer("Width", lexer)?;
        let height = dict.expect_integer("Height", lexer)?;
        let transfer_function = dict
            .get_object("TransferFunction", lexer)?
            .map(|obj| TransferFunction::from_obj(obj))
            .transpose()?;

        Ok(HalftoneSix {
            width,
            height,
            transfer_function,
        })
    }
}

#[derive(Debug)]
pub struct HalftoneTen {
    /// The side of square X, in device pixels
    x_square: i32,

    /// The side of square Y, in device pixels
    y_square: i32,

    transfer_function: Option<TransferFunction>,
}

impl HalftoneTen {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let x_square = dict.expect_integer("Xsquare", lexer)?;
        let y_square = dict.expect_integer("Ysquare", lexer)?;
        let transfer_function = dict
            .get_object("TransferFunction", lexer)?
            .map(|obj| TransferFunction::from_obj(obj))
            .transpose()?;

        Ok(Self {
            x_square,
            y_square,
            transfer_function,
        })
    }
}

#[derive(Debug)]
pub struct HalftoneSixteen {
    /// The width of the first (or only) rectangle in the threshold array, in device pixels.
    width: i32,

    /// The height of the first (or only) rectangle in the threshold array, in device pixels.
    height: i32,

    /// The width of the optional second rectangle in the threshold array, in device pixels.
    ///
    /// If this entry is present, the Height2 entry shall be present as well.
    /// If this entry is absent, the Height2 entry shall also be absent, and the threshold array has
    /// only one rectangle
    width_two: Option<i32>,

    /// The height of the optional second rectangle in the threshold array, in device pixels
    height_two: Option<i32>,

    transfer_function: Option<TransferFunction>,
}

impl HalftoneSixteen {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let width = dict.expect_integer("Width", lexer)?;
        let height = dict.expect_integer("Height", lexer)?;
        let width_two = dict.get_integer("Width2", lexer)?;
        let height_two = dict.get_integer("Height2", lexer)?;
        let transfer_function = dict
            .get_object("TransferFunction", lexer)?
            .map(|obj| TransferFunction::from_obj(obj))
            .transpose()?;

        Ok(Self {
            width,
            height,
            width_two,
            height_two,
            transfer_function,
        })
    }
}

impl HalftoneDictionary {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        dict.expect_type("Halftone", lexer, false)?;

        Ok(
            match HalftoneType::from_integer(dict.expect_integer("HalftoneType", lexer)?)? {
                HalftoneType::One => HalftoneDictionary::One(HalftoneOne::from_dict(dict, lexer)?),
                HalftoneType::Five => {
                    HalftoneDictionary::Five(HalftoneFive::from_dict(dict, lexer)?)
                }
                HalftoneType::Six => HalftoneDictionary::Six(HalftoneSix::from_dict(dict, lexer)?),
                HalftoneType::Ten => HalftoneDictionary::Ten(HalftoneTen::from_dict(dict, lexer)?),
                HalftoneType::Sixteen => {
                    HalftoneDictionary::Sixteen(HalftoneSixteen::from_dict(dict, lexer)?)
                }
            },
        )
    }
}
