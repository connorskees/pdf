use crate::stream::Stream;

/// Type 0 functions use a sequence of sample values (contained in a stream) to provide an
/// approximation for functions whose domains and ranges are bounded. The samples are organized
/// as an m-dimensional table in which each entry has n components.
#[derive(Debug, Clone, FromObj)]
pub struct SampledFunction<'a> {
    /// An array of m positive integers that shall specify the number of samples in each
    /// input dimension of the sample table
    #[field("Size")]
    size: Vec<u32>,

    /// The number of bits that shall represent each sample. (If the function has multiple
    /// output values, each one shall occupy BitsPerSample bits.)
    #[field("BitsPerSample")]
    bits_per_sample: BitsPerSample,

    /// The order of interpolation between samples. Valid values shall be 1 and 3, specifying
    /// linear and cubic spline interpolation, respectively
    ///
    /// Default value: 1
    #[field("Order", default = InterpolationOrder::default())]
    order: InterpolationOrder,

    /// An array of 2 * m numbers specifying the linear mapping of input values into the domain
    /// of the function's sample table.
    ///
    /// Default value: [0 (Size0 - 1) 0 (Size1 - 1) ...]
    #[field(
        "Encode",
        default = size.iter().flat_map(|&i| vec![0.0, (i as f32) - 1.0]).collect()
    )]
    encode: Vec<f32>,

    /// An array of 2 * n numbers specifying the linear mapping of sample values into the range
    /// appropriate for the function's output values
    ///
    /// Default value: same as the value of Range
    // todo: doesn't have to be Option because of default
    #[field("Decode")]
    decode: Option<Vec<f32>>,

    #[field]
    stream: Stream<'a>,
}

#[pdf_enum(Integer)]
#[derive(Default)]
enum InterpolationOrder {
    #[default]
    Linear = 1,
    Cubic = 3,
}

#[pdf_enum(Integer)]
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
