use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType},
    stream::Stream,
    FromObj, Resolve,
};

use self::{
    exponential_interpolation::ExponentialInterpolationFunction,
    postscript_calculator::PostScriptCalculatorFunction, sampled::SampledFunction,
    stitching::StitchingFunction,
};

mod exponential_interpolation;
mod postscript_calculator;
mod sampled;
mod stitching;

#[derive(Debug, Clone)]
pub struct Function<'a> {
    /// An array of 2 * m numbers, where m shall be the number of input values.
    /// For each i from 0 to m - 1, Domain2i shall be less than or equal to Domain2i+1,
    /// and the ith input value, xi, shall lie in the interval Domain2i <= xi <= Domain2i+1.
    /// Input values outside the declared domain shall be clipped to the nearest boundary
    /// value.
    domain: Vec<f32>,

    /// An array of 2 * n numbers, where n shall be the number of output values. For
    /// each j from 0 to n - 1, Range2j shall be less than or equal to Range2j+1,
    /// and the jth output value, yj , shall lie in the interval Range2j <= yj <= Range2j+1.
    /// Output values outside the declared range shall be clipped to the nearest
    /// boundary value. If this entry is absent, no clipping shall be done.
    // todo: optional for type 0 and type 4
    range: Option<Vec<f32>>,

    subtype: FunctionSubtype<'a>,
}

#[derive(Debug)]
pub(crate) enum StreamOrDict<'a> {
    Stream(Stream<'a>),
    Dict(Dictionary<'a>),
}

impl<'a> StreamOrDict<'a> {
    pub fn dict(&mut self) -> &mut Dictionary<'a> {
        match self {
            Self::Dict(dict) => dict,
            Self::Stream(stream) => &mut stream.dict.other,
        }
    }

    pub fn expect_stream(self) -> PdfResult<Stream<'a>> {
        match self {
            Self::Dict(..) => anyhow::bail!(ParseError::MismatchedObjectType {
                expected: ObjectType::Stream,
            }),
            Self::Stream(stream) => Ok(stream),
        }
    }

    pub fn into_obj(self) -> Object<'a> {
        match self {
            Self::Dict(dict) => Object::Dictionary(dict),
            Self::Stream(stream) => Object::Stream(stream),
        }
    }
}

impl<'a> FromObj<'a> for Function<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        let mut stream_or_dict = if let Ok(stream) = resolver.assert_stream(obj.clone()) {
            StreamOrDict::Stream(stream)
        } else {
            StreamOrDict::Dict(resolver.assert_dict(obj)?)
        };

        let dict = stream_or_dict.dict();

        let domain = dict.expect("Domain", resolver)?;
        let range = dict.get("Range", resolver)?;

        let subtype = FunctionSubtype::from_obj(stream_or_dict.into_obj(), resolver)?;

        Ok(Self {
            domain,
            range,
            subtype,
        })
    }
}

#[derive(Debug, Clone)]
enum FunctionSubtype<'a> {
    Sampled(SampledFunction<'a>),
    ExponentialInterpolation(ExponentialInterpolationFunction),
    Stitching(StitchingFunction<'a>),
    PostScriptCalculator(PostScriptCalculatorFunction),
}

impl<'a> FromObj<'a> for FunctionSubtype<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut stream_or_dict = match resolver.resolve(obj)? {
            Object::Stream(stream) => StreamOrDict::Stream(stream),
            Object::Dictionary(dict) => StreamOrDict::Dict(dict),
            _ => todo!(),
        };

        let dict = stream_or_dict.dict();
        let subtype = FunctionType::from_integer(dict.expect_integer("FunctionType", resolver)?)?;

        Ok(match subtype {
            FunctionType::Sampled => FunctionSubtype::Sampled(SampledFunction::from_obj(
                Object::Stream(stream_or_dict.expect_stream()?),
                resolver,
            )?),
            FunctionType::ExponentialInterpolation => FunctionSubtype::ExponentialInterpolation(
                ExponentialInterpolationFunction::from_dict(dict, resolver)?,
            ),
            FunctionType::Stitching => {
                FunctionSubtype::Stitching(StitchingFunction::from_dict(dict, resolver)?)
            }
            FunctionType::PostScriptCalculator => {
                FunctionSubtype::PostScriptCalculator(PostScriptCalculatorFunction::from_stream(
                    stream_or_dict.expect_stream()?,
                    resolver,
                )?)
            }
        })
    }
}

#[pdf_enum(Integer)]
enum FunctionType {
    Sampled = 0,
    ExponentialInterpolation = 2,
    Stitching = 3,
    PostScriptCalculator = 4,
}

#[derive(Debug, Clone)]
pub enum SpotFunction<'a> {
    Predefined(PredefinedSpotFunction),
    Function(Function<'a>),
}

impl<'a> FromObj<'a> for SpotFunction<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(if let Object::Name(ref name) = obj {
            SpotFunction::Predefined(PredefinedSpotFunction::from_str(name)?)
        } else {
            SpotFunction::Function(Function::from_obj(obj, resolver)?)
        })
    }
}

#[pdf_enum]
pub enum PredefinedSpotFunction {
    SimpleDot = "SimpleDot",
    InvertedSimpleDot = "InvertedSimpleDot",
    DoubleDot = "DoubleDot",
    InvertedDoubleDot = "InvertedDoubleDot",
    CosineDot = "CosineDot",
    Double = "Double",
    Line = "Line",
    LineX = "LineX",
    LineY = "LineY",
    Round = "Round",
    Ellipse = "Ellipse",
    EllipseA = "EllipseA",
    InvertedEllipseA = "InvertedEllipseA",
    EllipseB = "EllipseB",
    EllipseC = "EllipseC",
    InvertedEllipseC = "InvertedEllipseC",
    Square = "Square",
    Cross = "Cross",
    Rhomboid = "Rhomboid",
    Diamond = "Diamond",
}

#[derive(Debug, Clone)]
pub enum TransferFunction<'a> {
    Identity,
    Default,
    Single(Function<'a>),
    Colorants {
        a: Function<'a>,
        b: Function<'a>,
        c: Function<'a>,
        d: Function<'a>,
    },
}

impl<'a> FromObj<'a> for TransferFunction<'a> {
    // todo: array, default
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        // todo: dont use this
        Ok(if obj.name_is("Identity") {
            TransferFunction::Identity
        } else {
            TransferFunction::Single(Function::from_obj(obj, resolver)?)
        })
    }
}
