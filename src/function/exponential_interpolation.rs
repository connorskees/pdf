use crate::{error::PdfResult, objects::Dictionary, Resolve};

/// Type 2 functions (PDF 1.3) include a set of parameters that define an exponential
/// interpolation of one input value and n output values
#[derive(Debug, Clone)]
pub struct ExponentialInterpolationFunction {
    /// An array of n numbers that shall define the function result when x = 0.0.
    ///
    /// Default value: [0.0]
    c0: Vec<f32>,

    /// An array of n numbers that shall define the function result when x = 1.0.
    ///
    /// Default value: [1.0]
    c1: Vec<f32>,

    /// The interpolation exponent. Each input value x shall return n values, given by
    /// yj = C0j + xN * (C1j - C0j), for 0 <= j < n
    n: f32,
}

impl ExponentialInterpolationFunction {
    pub fn from_dict<'a>(
        dict: &mut Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let c0 = dict
            .get_arr("C0", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?
            .unwrap_or_else(|| vec![0.0]);

        let c1 = dict
            .get_arr("C1", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?
            .unwrap_or_else(|| vec![1.0]);

        let n = dict.expect_number("N", resolver)?;

        Ok(Self { c0, c1, n })
    }
}
