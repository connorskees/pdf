use crate::{error::PdfResult, objects::Dictionary, FromObj, Resolve};

use super::Function;

/// Type 3 functions (PDF 1.3) define a stitching of the subdomains of several 1-input functions to
/// produce a single new 1-input function. Since the resulting stitching function is a 1-input function,
/// the domain is given by a twoelement array, [Domain0 Domain1].
#[derive(Debug, Clone, FromObj)]
pub struct StitchingFunction<'a> {
    /// An array of k 1-input functions that shall make up the stitching function. The output
    /// dimensionality of all functions shall be the same, and compatible with the value of Range if Range
    /// is present
    #[field("Functions")]
    functions: Vec<Function<'a>>,

    /// An array of k - 1 numbers that, in combination with Domain, shall define the intervals to which
    /// each function from the Functions array shall apply. Bounds elements shall be in order of
    /// increasing value, and each value shall be within the domain defined by Domain
    #[field("Bounds")]
    bounds: Vec<f32>,

    /// An array of 2 * k numbers that, taken in pairs, shall map each subset of the domain defined by
    /// Domain and the Bounds array to the domain of the corresponding function
    #[field("Encode")]
    encode: Vec<f32>,
}

impl<'a> StitchingFunction<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let functions = dict
            .expect_arr("Functions", resolver)?
            .into_iter()
            .map(|obj| Function::from_obj(obj, resolver))
            .collect::<PdfResult<Vec<Function>>>()?;

        let bounds = dict
            .expect_arr("Bounds", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;

        let encode = dict
            .expect_arr("Encode", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;

        Ok(Self {
            functions,
            bounds,
            encode,
        })
    }
}
