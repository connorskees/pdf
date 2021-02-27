use crate::{error::PdfResult, objects::Object, pdf_enum};

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

pdf_enum!(
    #[derive(Debug)]
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
