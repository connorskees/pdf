use crate::{error::PdfResult, objects::Dictionary, Resolve};

#[derive(Debug)]
pub struct NameTree;

impl NameTree {
    pub fn from_dict<'a>(_dict: Dictionary, _resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        todo!()
    }
}
