use crate::{error::PdfResult, objects::Dictionary, Resolve};

#[derive(Debug)]
pub struct NameTree;

impl NameTree {
    pub fn from_dict(_dict: Dictionary, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}
