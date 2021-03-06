use crate::{
    error::PdfResult, filter::flate::BitsPerComponent, function::Function, objects::Dictionary,
    Resolve,
};

use super::freeform::{BitsPerCoordinate, BitsPerFlag};

/// Type 6 shadings (Coons patch meshes) are constructed from one or more colour patches,
/// each bounded by four cubic Bézier curves. Degenerate Bézier curves are allowed and are
/// useful for certain graphical effects. At least one complete patch shall be specified.
///
/// A Coons patch generally has two independent aspects:
///   * Colours are specified for each corner of the unit square, and bilinear interpolation
///     is used to fill in colours over the entire unit square
///   * Coordinates are mapped from the unit square into a four-sided patch whose sides are
///     not necessarily linear. The mapping is continuous: the corners of the unit square map
///     to corners of the patch and the sides of the unit square map to sides of the patch
#[derive(Debug)]
pub struct CoonsPatchMeshShading {
    /// The number of bits used to represent each vertex coordinate.
    ///
    /// The value shall be 1, 2, 4, 8, 12, 16, 24, or 32.
    bits_per_coordinate: BitsPerCoordinate,

    /// The number of bits used to represent each colour component.
    ///
    /// The value shall be 1, 2, 4, 8, 12, or 16.
    bits_per_component: BitsPerComponent,

    /// The number of bits used to represent the edge flag for each vertex.
    /// The value of BitsPerFlag shall be 2, 4, or 8, but only the least
    /// significant 2 bits in each flag value shall be used. The value for
    /// the edge flag shall be 0, 1, or 2.
    bits_per_flag: BitsPerFlag,

    /// An array of numbers specifying how to map coordinates and colour components into the
    /// appropriate ranges of values. The decoding method is similar to that used in image
    /// dictionaries. The ranges shall be specified as follows:
    ///
    /// [xmin xmax ymin ymax c1,min c1,max … cn,min cn,max]
    ///
    /// Only one pair of c values shall be specified if a Function entry is present
    decode: Vec<f32>,

    /// A 1-in, n-out function or an array of n 1-in, 1-out functions (where n is
    /// the number of colour components in the shading dictionary’s colour space).
    /// If this entry is present, the colour data for each vertex shall be specified
    /// by a single parametric variable rather than by n separate colour components.
    /// The designated function(s) shall be called with each interpolated value of
    /// the parametric variable to determine the actual colour at each point. Each
    /// input value shall be forced into the range interval specified for the corresponding
    /// colour component in the shading dictionary’s Decode array. Each function’s
    /// domain shall be a superset of that interval. If the value returned by the
    /// function for a given colour component is out of range, it shall be adjusted
    /// to the nearest valid value.
    ///
    /// This entry shall not be used with an Indexed colour space
    function: Option<Function>,
}

impl CoonsPatchMeshShading {
    pub fn from_dict(dict: &mut Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let bits_per_coordinate =
            BitsPerCoordinate::from_integer(dict.expect_integer("BitsPerCoordinate", resolver)?)?;
        let bits_per_component =
            BitsPerComponent::from_integer(dict.expect_integer("BitsPerComponent", resolver)?)?;
        let bits_per_flag =
            BitsPerFlag::from_integer(dict.expect_integer("BitsPerFlag", resolver)?)?;
        let decode = dict
            .expect_arr("Decode", resolver)?
            .into_iter()
            .map(|obj| resolver.assert_number(obj))
            .collect::<PdfResult<Vec<f32>>>()?;
        let function = dict.get_function("Function", resolver)?;

        Ok(Self {
            bits_per_coordinate,
            bits_per_component,
            bits_per_flag,
            decode,
            function,
        })
    }
}
