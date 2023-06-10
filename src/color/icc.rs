use crate::{catalog::MetadataStream, stream::Stream};

use super::ColorSpace;

#[derive(Debug, Clone, FromObj)]
pub struct IccStream<'a> {
    /// The number of colour components in the colour space described by the ICC
    /// profile data. This number shall match the number of components
    /// actually in the ICC profile.
    #[field("N")]
    pub num_of_color_components: i32,

    /// An alternate colour space that shall be used in case the one specified in
    /// the stream data is not supported.
    ///
    /// Non-conforming readers may use this colour space. The alternate space may
    /// be any valid colour space (except a Pattern colour space) that has
    /// the number of components specified by N. If this entry is omitted
    /// and the conforming reader does not understand the ICC profile data,
    /// the colour space that shall be used is DeviceGray, DeviceRGB, or
    /// DeviceCMYK, depending on whether the value of N is 1, 3, or 4,
    /// respectively.
    ///
    /// There shall not be conversion of source colour values, such as a tint
    /// transformation, when using the alternate colour space. Colour
    /// values within the range of the ICCBased colour space might not be
    /// within the range of the alternate colour space. In this case, the
    /// nearest values within the range of the alternate space shall be
    /// substituted.
    #[field("Alternate")]
    pub alternate: Option<Box<ColorSpace<'a>>>,

    /// An array of 2 × N numbers [min0 max0 min1 max1 ...] that shall specify the
    /// minimum and maximum valid values of the corresponding colour components.
    /// These values shall match the information in the ICC profile.
    ///
    /// Default value: [0.0 1.0 0.0 1.0 ...]
    // todo: prettier way to do this?
    // todo: special struct for this
    #[field("Range", default = [0.0_f32, 1.0].into_iter().cycle().take(num_of_color_components as usize * 2).collect())]
    pub range: Vec<f32>,

    /// A metadata stream that shall contain metadata for the colour space
    #[field("Metadata")]
    pub metadata: Option<MetadataStream<'a>>,

    #[field]
    pub stream: Stream<'a>,
}

#[pdf_enum(Integer)]
enum IccColorComponentNum {
    One = 1,
    Three = 3,
    Four = 4,
}
