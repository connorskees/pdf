use crate::{filter::flate::BitsPerComponent, function::Function, stream::Stream};

/// Type 4 shadings (free-form Gouraud-shaded triangle meshes) are commonly used to
/// represent complex coloured and shaded three-dimensional shapes. The area to be
/// shaded is defined by a path composed entirely of triangles. The colour at each
/// vertex of the triangles is specified, and a technique known as Gouraud interpolation
/// is used to colour the interiors. The interpolation functions defining the shading may
/// be linear or nonlinear
#[derive(Debug, Clone, FromObj)]
pub struct FreeformShading<'a> {
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

    /// An array of numbers specifying how to map vertex coordinates and colour
    /// components into the appropriate ranges of values. The decoding method is
    /// similar to that used in image dictionaries. The ranges shall be specified
    /// as follows:
    ///
    /// [xmin xmax ymin ymax c1,min c1,max ... cn,min cn,max]
    ///
    /// Only one pair of c values shall be specified if a Function entry is present
    #[field("Decode")]
    decode: Vec<f32>,

    #[field("Function")]
    function: Option<Function<'a>>,

    #[field]
    stream: Stream<'a>,
}

#[pdf_enum(Integer)]
pub enum BitsPerCoordinate {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
}

#[pdf_enum(Integer)]
pub enum BitsPerFlag {
    Two = 2,
    Four = 4,
    Eight = 8,
}
