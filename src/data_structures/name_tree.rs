use crate::{error::PdfResult, objects::{ Object}, FromObj, Resolve};

#[derive(Debug)]
pub struct NameTree;

impl<'a> FromObj<'a> for NameTree {
    fn from_obj(_obj: Object<'a>, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        todo!()
    }
}
