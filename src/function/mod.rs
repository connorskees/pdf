use crate::{
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType},
    pdf_enum,
    stream::Stream,
    Resolve,
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
pub struct Function {
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

    subtype: FunctionSubtype,
}

#[derive(Debug)]
pub(crate) enum StreamOrDict {
    Stream(Stream),
    Dict(Dictionary),
}

impl StreamOrDict {
    pub fn dict(&mut self) -> &mut Dictionary {
        match self {
            Self::Dict(dict) => dict,
            Self::Stream(stream) => &mut stream.dict.other,
        }
    }

    pub fn expect_stream(self) -> PdfResult<Stream> {
        match self {
            Self::Dict(dict) => Err(ParseError::MismatchedObjectType {
                expected: ObjectType::Stream,
                found: Object::Dictionary(dict),
            }),
            Self::Stream(stream) => Ok(stream),
        }
    }
}

impl Function {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        let mut stream_or_dict = if let Ok(stream) = resolver.assert_stream(obj.clone()) {
            StreamOrDict::Stream(stream)
        } else {
            StreamOrDict::Dict(resolver.assert_dict(obj)?)
        };

        let dict = stream_or_dict.dict();

        let domain = dict
            .expect_arr("Domain", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;
        let range = dict
            .get_arr("Range", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?;

        let subtype = FunctionSubtype::from_stream_or_dict(stream_or_dict, resolver)?;

        Ok(Self {
            domain,
            range,
            subtype,
        })
    }
}

#[derive(Debug, Clone)]
enum FunctionSubtype {
    Sampled(SampledFunction),
    ExponentialInterpolation(ExponentialInterpolationFunction),
    Stitching(StitchingFunction),
    PostScriptCalculator(PostScriptCalculatorFunction),
}

impl FunctionSubtype {
    pub fn from_stream_or_dict(
        mut stream_or_dict: StreamOrDict,
        resolver: &mut dyn Resolve,
    ) -> PdfResult<Self> {
        let dict = stream_or_dict.dict();
        let subtype = FunctionType::from_integer(dict.expect_integer("FunctionType", resolver)?)?;

        Ok(match subtype {
            FunctionType::Sampled => FunctionSubtype::Sampled(SampledFunction::from_stream(
                stream_or_dict.expect_stream()?,
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

pdf_enum!(
    int
    #[derive(Debug, Clone, Copy)]
    enum FunctionType {
        Sampled = 0,
        ExponentialInterpolation = 2,
        Stitching = 3,
        PostScriptCalculator = 4,
    }
);

#[derive(Debug, Clone)]
pub enum SpotFunction {
    Predefined(PredefinedSpotFunction),
    Function(Function),
}

impl SpotFunction {
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        Ok(if let Object::Name(ref name) = obj {
            SpotFunction::Predefined(PredefinedSpotFunction::from_str(name)?)
        } else {
            SpotFunction::Function(Function::from_obj(obj, resolver)?)
        })
    }
}

pdf_enum!(
    #[derive(Debug, Clone, Copy)]
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
);

#[derive(Debug, Clone)]
pub enum TransferFunction {
    Identity,
    Default,
    Single(Function),
    Colorants {
        a: Function,
        b: Function,
        c: Function,
        d: Function,
    },
}

impl TransferFunction {
    // todo: array, default
    pub fn from_obj(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        // todo: dont use this
        Ok(if obj.name_is("Identity") {
            TransferFunction::Identity
        } else {
            TransferFunction::Single(Function::from_obj(obj, resolver)?)
        })
    }
}
