use crate::{
    color::ColorSpace,
    error::{ParseError, PdfResult},
    function::{SpotFunction, TransferFunction},
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    FromObj, Resolve,
};

#[derive(Debug, Clone)]
pub enum Halftones<'a> {
    Default,
    Dictionary(HalftoneDictionary<'a>),
    Stream(HalftoneStream),
}

impl<'a> FromObj<'a> for Halftones<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(match obj {
            Object::Name(name) if name == "Default" => Halftones::Default,
            Object::Stream(stream) => Halftones::Stream(HalftoneStream::from_stream(stream)?),
            Object::Dictionary(dict) => {
                Halftones::Dictionary(HalftoneDictionary::from_dict(dict, resolver)?)
            }
            _ => {
                anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Name, ObjectType::Stream, ObjectType::Dictionary],
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

// todo: pdf_enum?
#[pdf_enum(Integer)]
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

#[derive(Debug, Clone, FromObj)]
pub struct HalftoneOne<'a> {
    /// The screen frequency, measured in halftone cells per inch in device space
    #[field("Frequency")]
    frequency: f32,

    /// The screen angle, in degrees of rotation counterclockwise with respect to
    /// the device coordinate system
    #[field("Angle")]
    angle: f32,

    /// A function object defining the order in which device pixels within a screen
    /// cell shall be adjusted for different gray levels, or the name of one of the
    /// predefined spot functions
    #[field("SpotFunction")]
    spot_function: SpotFunction<'a>,

    /// A flag specifying whether to invoke a special halftone algorithm that is extremely
    /// precise but computationally expensive; see Note 1 for further discussion.
    ///
    /// Default value: false
    #[field("AccurateScreens")]
    accurate_screens: Option<bool>,

    /// A transfer function, which overrides the current transfer function in the graphics
    /// state for the same component.
    ///
    /// This entry shall be present if the dictionary is a component of a type 5 halftone and
    /// represents either a nonprimary or nonstandard primary colour component.
    ///
    /// The name Identity may be used to specify the identity function
    #[field("TransferFunction")]
    transfer_function: Option<TransferFunction<'a>>,
}

#[derive(Debug, Clone)]
pub struct HalftoneFive<'a> {
    colorant: ColorSpace<'a>,
    default: Box<Halftones<'a>>,
}

impl<'a> FromObj<'a> for HalftoneFive<'a> {
    fn from_obj(_obj: Object<'a>, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, FromObj)]
pub struct HalftoneSix<'a> {
    #[field("Width")]
    width: i32,
    #[field("Height")]
    height: i32,
    #[field("TransferFunction")]
    transfer_function: Option<TransferFunction<'a>>,
}

#[derive(Debug, Clone, FromObj)]
pub struct HalftoneTen<'a> {
    /// The side of square X, in device pixels
    #[field("Xsquare")]
    x_square: i32,

    /// The side of square Y, in device pixels
    #[field("Ysquare")]
    y_square: i32,

    #[field("TransferFunction")]
    transfer_function: Option<TransferFunction<'a>>,
}

#[derive(Debug, Clone, FromObj)]
pub struct HalftoneSixteen<'a> {
    /// The width of the first (or only) rectangle in the threshold array, in device pixels.
    #[field("Width")]
    width: i32,

    /// The height of the first (or only) rectangle in the threshold array, in device pixels.
    #[field("Height")]
    height: i32,

    /// The width of the optional second rectangle in the threshold array, in device pixels.
    ///
    /// If this entry is present, the Height2 entry shall be present as well.
    /// If this entry is absent, the Height2 entry shall also be absent, and the threshold array has
    /// only one rectangle
    #[field("Width2")]
    width_two: Option<i32>,

    /// The height of the optional second rectangle in the threshold array, in device pixels
    #[field("Height2")]
    height_two: Option<i32>,

    #[field("TransferFunction")]
    transfer_function: Option<TransferFunction<'a>>,
}

impl<'a> HalftoneDictionary<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        dict.expect_type("Halftone", resolver, false)?;

        let halftone_type = dict.expect::<HalftoneType>("HalftoneType", resolver)?;

        let obj = Object::Dictionary(dict);

        Ok(match halftone_type {
            HalftoneType::One => HalftoneDictionary::One(HalftoneOne::from_obj(obj, resolver)?),
            HalftoneType::Five => HalftoneDictionary::Five(HalftoneFive::from_obj(obj, resolver)?),
            HalftoneType::Six => HalftoneDictionary::Six(HalftoneSix::from_obj(obj, resolver)?),
            HalftoneType::Ten => HalftoneDictionary::Ten(HalftoneTen::from_obj(obj, resolver)?),
            HalftoneType::Sixteen => {
                HalftoneDictionary::Sixteen(HalftoneSixteen::from_obj(obj, resolver)?)
            }
        })
    }
}
