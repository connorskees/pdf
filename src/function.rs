use crate::{
    error::{ParseError, PdfResult},
    objects::Object,
};

#[derive(Debug)]
pub struct Function;

impl Function {
    pub fn from_obj(_obj: Object) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub enum SpotFunction {
    Predefined(PredefinedSpotFunction),
    Function(Function),
}

impl SpotFunction {
    pub fn from_obj(obj: Object) -> PdfResult<Self> {
        Ok(if let Object::Name(ref name) = obj {
            SpotFunction::Predefined(PredefinedSpotFunction::from_str(name)?)
        } else {
            SpotFunction::Function(Function::from_obj(obj)?)
        })
    }
}

#[derive(Debug)]
pub enum PredefinedSpotFunction {
    SimpleDot,
    InvertedSimpleDot,
    DoubleDot,
    InvertedDoubleDot,
    CosineDot,
    Double,
    Line,
    LineX,
    LineY,
    Round,
    Ellipse,
    EllipseA,
    InvertedEllipseA,
    EllipseB,
    EllipseC,
    InvertedEllipseC,
    Square,
    Cross,
    Rhomboid,
    Diamond,
}

impl PredefinedSpotFunction {
    pub fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "SimpleDot" => Self::SimpleDot,
            "InvertedSimpleDot" => Self::InvertedSimpleDot,
            "DoubleDot" => Self::DoubleDot,
            "InvertedDoubleDot" => Self::InvertedDoubleDot,
            "CosineDot" => Self::CosineDot,
            "Double" => Self::Double,
            "Line" => Self::Line,
            "LineX" => Self::LineX,
            "LineY" => Self::LineY,
            "Round" => Self::Round,
            "Ellipse" => Self::Ellipse,
            "EllipseA" => Self::EllipseA,
            "InvertedEllipseA" => Self::InvertedEllipseA,
            "EllipseB" => Self::EllipseB,
            "EllipseC" => Self::EllipseC,
            "InvertedEllipseC" => Self::InvertedEllipseC,
            "Square" => Self::Square,
            "Cross" => Self::Cross,
            "Rhomboid" => Self::Rhomboid,
            "Diamond" => Self::Diamond,
            found => {
                return Err(ParseError::UnrecognizedVariant {
                    found: found.to_owned(),
                    ty: "PredefinedSpotFunction",
                })
            }
        })
    }
}

#[derive(Debug)]
pub enum TransferFunction {
    Identity,
    Function(Function),
}

impl TransferFunction {
    pub fn from_obj(obj: Object) -> PdfResult<Self> {
        Ok(if obj.name_is("Identity") {
            TransferFunction::Identity
        } else {
            TransferFunction::Function(Function::from_obj(obj)?)
        })
    }
}
