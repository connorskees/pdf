use crate::{filter::flate::BitsPerComponent, function::Function, stream::Stream};

use super::freeform::{BitsPerCoordinate, BitsPerFlag};

/// Type 7 shadings (tensor-product patch meshes) are identical to type 6, except that
/// they are based on a bicubic tensor-product patch defined by 16 control points instead
/// of the 12 control points that define a Coons patch. The shading dictionaries representing
/// the two patch types differ only in the value of the ShadingType entry and in the number
/// of control points specified for each patch in the data stream
#[derive(Debug, Clone, FromObj)]
pub struct TensorProductPatchMeshShading<'a> {
    /// The number of bits used to represent each vertex coordinate.
    ///
    /// The value shall be 1, 2, 4, 8, 12, 16, 24, or 32.
    #[field("BitsPerCoordinate")]
    bits_per_coordinate: BitsPerCoordinate,

    /// The number of bits used to represent each colour component.
    ///
    /// The value shall be 1, 2, 4, 8, 12, or 16.
    #[field("BitsPerComponent")]
    bits_per_component: BitsPerComponent,

    /// The number of bits used to represent the edge flag for each vertex.
    /// The value of BitsPerFlag shall be 2, 4, or 8, but only the least
    /// significant 2 bits in each flag value shall be used. The value for
    /// the edge flag shall be 0, 1, or 2.
    #[field("BitsPerFlag")]
    bits_per_flag: BitsPerFlag,

    /// An array of numbers specifying how to map coordinates and colour components into the
    /// appropriate ranges of values. The decoding method is similar to that used in image
    /// dictionaries. The ranges shall be specified as follows:
    ///
    /// [xmin xmax ymin ymax c1,min c1,max ... cn,min cn,max]
    ///
    /// Only one pair of c values shall be specified if a Function entry is present
    #[field("Decode")]
    decode: Vec<f32>,

    /// A 1-in, n-out function or an array of n 1-in, 1-out functions (where n is
    /// the number of colour components in the shading dictionary's colour space).
    /// If this entry is present, the colour data for each vertex shall be specified
    /// by a single parametric variable rather than by n separate colour components.
    /// The designated function(s) shall be called with each interpolated value of
    /// the parametric variable to determine the actual colour at each point. Each
    /// input value shall be forced into the range interval specified for the corresponding
    /// colour component in the shading dictionary's Decode array. Each function's
    /// domain shall be a superset of that interval. If the value returned by the
    /// function for a given colour component is out of range, it shall be adjusted
    /// to the nearest valid value.
    ///
    /// This entry shall not be used with an Indexed colour space
    #[field("Function")]
    function: Option<Function<'a>>,

    #[field]
    stream: Stream<'a>,
}
