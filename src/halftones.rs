use crate::{
    catalog::ColorSpace,
    error::{ParseError, PdfResult},
    function::{SpotFunction, TransferFunction},
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    Resolve,
};

#[derive(Debug, Clone)]
pub enum Halftones<'a> {
    Default,
    Dictionary(HalftoneDictionary<'a>),
    Stream(HalftoneStream),
}

impl<'a> Halftones<'a> {
    pub fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match obj {
            Object::Name(name) if name == "Default" => Halftones::Default,
            Object::Stream(stream) => Halftones::Stream(HalftoneStream::from_stream(stream)?),
            Object::Dictionary(dict) => {
                Halftones::Dictionary(HalftoneDictionary::from_dict(dict, resolver)?)
            }
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Name, ObjectType::Stream, ObjectType::Dictionary],
                    // found,
                });
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct HalftoneStream {}

impl HalftoneStream {
    pub fn from_stream(_stream: Stream) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum HalftoneDictionary<'a> {
    One(HalftoneOne<'a>),
    Five(HalftoneFive<'a>),
    Six(HalftoneSix<'a>),
    Ten(HalftoneTen<'a>),
    Sixteen(HalftoneSixteen<'a>),
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

#[derive(Debug, Clone)]
pub struct HalftoneOne<'a> {
    /// The screen frequency, measured in halftone cells per inch in device space
    frequency: f32,

    /// The screen angle, in degrees of rotation counterclockwise with respect to
    /// the device coordinate system
    angle: f32,

    /// A function object defining the order in which device pixels within a screen
    /// cell shall be adjusted for different gray levels, or the name of one of the
    /// predefined spot functions
    spot_function: SpotFunction<'a>,

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
    transfer_function: Option<TransferFunction<'a>>,
}

impl<'a> HalftoneOne<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let frequency = dict.expect_number("Frequency", resolver)?;
        let angle = dict.expect_number("Angle", resolver)?;
        let spot_function =
            SpotFunction::from_obj(dict.expect_object("SpotFunction", resolver)?, resolver)?;
        let accurate_screens = dict.get_bool("AccurateScreens", resolver)?;
        let transfer_function = dict
            .get_object("TransferFunction", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
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

#[derive(Debug, Clone)]
pub struct HalftoneFive<'a> {
    colorant: ColorSpace,
    default: Box<Halftones<'a>>,
}

impl<'a> HalftoneFive<'a> {
    pub fn from_dict(mut _dict: Dictionary, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct HalftoneSix<'a> {
    width: i32,
    height: i32,
    transfer_function: Option<TransferFunction<'a>>,
}

impl<'a> HalftoneSix<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let width = dict.expect_integer("Width", resolver)?;
        let height = dict.expect_integer("Height", resolver)?;
        let transfer_function = dict
            .get_object("TransferFunction", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
            .transpose()?;

        Ok(HalftoneSix {
            width,
            height,
            transfer_function,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HalftoneTen<'a> {
    /// The side of square X, in device pixels
    x_square: i32,

    /// The side of square Y, in device pixels
    y_square: i32,

    transfer_function: Option<TransferFunction<'a>>,
}

impl<'a> HalftoneTen<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let x_square = dict.expect_integer("Xsquare", resolver)?;
        let y_square = dict.expect_integer("Ysquare", resolver)?;
        let transfer_function = dict
            .get_object("TransferFunction", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
            .transpose()?;

        Ok(Self {
            x_square,
            y_square,
            transfer_function,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HalftoneSixteen<'a> {
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

    transfer_function: Option<TransferFunction<'a>>,
}

impl<'a> HalftoneSixteen<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let width = dict.expect_integer("Width", resolver)?;
        let height = dict.expect_integer("Height", resolver)?;
        let width_two = dict.get_integer("Width2", resolver)?;
        let height_two = dict.get_integer("Height2", resolver)?;
        let transfer_function = dict
            .get_object("TransferFunction", resolver)?
            .map(|obj| TransferFunction::from_obj(obj, resolver))
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

impl<'a> HalftoneDictionary<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        dict.expect_type("Halftone", resolver, false)?;

        Ok(
            match HalftoneType::from_integer(dict.expect_integer("HalftoneType", resolver)?)? {
                HalftoneType::One => {
                    HalftoneDictionary::One(HalftoneOne::from_dict(dict, resolver)?)
                }
                HalftoneType::Five => {
                    HalftoneDictionary::Five(HalftoneFive::from_dict(dict, resolver)?)
                }
                HalftoneType::Six => {
                    HalftoneDictionary::Six(HalftoneSix::from_dict(dict, resolver)?)
                }
                HalftoneType::Ten => {
                    HalftoneDictionary::Ten(HalftoneTen::from_dict(dict, resolver)?)
                }
                HalftoneType::Sixteen => {
                    HalftoneDictionary::Sixteen(HalftoneSixteen::from_dict(dict, resolver)?)
                }
            },
        )
    }
}
