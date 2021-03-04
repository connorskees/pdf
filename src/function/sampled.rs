use crate::{error::PdfResult, pdf_enum, stream::Stream, Resolve};

/// Type 0 functions use a sequence of sample values (contained in a stream) to provide an
/// approximation for functions whose domains and ranges are bounded. The samples are organized
/// as an m-dimensional table in which each entry has n components.
#[derive(Debug)]
pub struct SampledFunction {
    /// An array of m positive integers that shall specify the number of samples in each
    /// input dimension of the sample table
    size: Vec<u32>,

    /// The number of bits that shall represent each sample. (If the function has multiple
    /// output values, each one shall occupy BitsPerSample bits.)
    bits_per_sample: BitsPerSample,

    /// The order of interpolation between samples. Valid values shall be 1 and 3, specifying
    /// linear and cubic spline interpolation, respectively
    ///
    /// Default value: 1
    order: InterpolationOrder,

    /// An array of 2 × m numbers specifying the linear mapping of input values into the domain
    /// of the function’s sample table.
    ///
    /// Default value: [0 (Size0 − 1) 0 (Size1 − 1) ...]
    encode: Vec<f32>,

    /// An array of 2 × n numbers specifying the linear mapping of sample values into the range
    /// appropriate for the function’s output values
    ///
    /// Default value: same as the value of Range
    // todo: doesn't have to be Option because of default
    decode: Option<Vec<f32>>,

    stream: Stream,
}

impl SampledFunction {
    pub fn from_stream(mut stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;
        let size = dict
            .expect_arr("Size", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_unsigned_integer(obj))
            .collect::<PdfResult<Vec<u32>>>()?;

        let bits_per_sample =
            BitsPerSample::from_integer(dict.expect_integer("BitsPerSample", resolver)?)?;
        let order = InterpolationOrder::from_integer(dict.expect_integer("Order", resolver)?)?;
        let encode = dict
            .get_arr("Encode", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?
            .unwrap_or_else(|| {
                size.iter()
                    .flat_map(|&i| vec![0.0, (i as f32) - 1.0])
                    .collect()
            });
        let decode = dict
            .get_arr("Decode", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?;

        Ok(Self {
            size,
            bits_per_sample,
            order,
            encode,
            decode,
            stream,
        })
    }
}

pdf_enum!(
    int
    #[derive(Debug)]
    enum InterpolationOrder {
        Linear = 1,
        Cubic = 3,
    }
);

impl Default for InterpolationOrder {
    fn default() -> Self {
        Self::Linear
    }
}

pdf_enum!(
    int
    #[derive(Debug, Clone, Copy)]
    enum BitsPerSample {
        One = 1,
        Two = 2,
        Four = 4,
        Eight = 8,
        Twelve = 12,
        Sixteen = 16,
        TwentyFour = 24,
        ThirtyTwo = 32,
    }
);

//1, 2, 4, 8, 12, 16, 24, and 32
