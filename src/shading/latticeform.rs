use crate::{filter::flate::BitsPerComponent, function::Function, stream::Stream};

use super::freeform::BitsPerCoordinate;

/// Type 5 shadings (lattice-form Gouraud-shaded triangle meshes) are similar to type
/// 4, but instead of using freeform geometry, their vertices are arranged in a
/// pseudorectangular lattice, which is topologically equivalent to a rectangular
/// grid. The vertices are organized into rows, which need not be geometrically linear
#[derive(Debug, Clone, FromObj)]
pub struct LatticeformShading<'a> {
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

    /// The number of vertices in each row of the lattice; the value shall be
    /// greater than or equal to 2. The number of rows need not be specified.
    #[field("BitsPerFlag")]
    vertices_per_row: u32,

    /// An array of numbers specifying how to map vertex coordinates and colour
    /// components into the appropriate ranges of values. The decoding method is
    /// similar to that used in image dictionaries. The ranges shall be specified
    /// as follows:
    ///
    /// [xmin xmax ymin ymax c1,min c1,max ... cn,min cn,max]
    ///
    /// Only one pair of c values shall be specified if a Function entry is
    /// present
    #[field("Decode")]
    decode: Vec<f32>,

    /// A 1-in, n-out function or an array of n 1-in, 1-out functions (where n is
    /// the number of colour components in the shading dictionary's colour space).
    /// If this entry is present, the colour data for each vertex shall be specified
    /// by a single parametric variable rather than by n separate colour components.
    /// The designated function(s) shall be called with each interpolated value of
    /// the parametric variable to determine the actual colour at each point. Each
    /// input value shall be forced into the range interval specified for the
    /// corresponding colour component in the shading dictionary's Decode array.
    /// Each function's domain shall be a superset of that interval. If the value
    /// returned by the function for a given colour component is out of range, it
    /// shall be adjusted to the nearest valid value.
    ///
    /// This entry shall not be used with an Indexed colour space.
    #[field("Function")]
    function: Option<Function<'a>>,

    #[field]
    stream: Stream<'a>,
}
