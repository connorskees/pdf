use crate::{
    catalog::assert_len, data_structures::Matrix, error::PdfResult, function::Function,
    objects::Object, FromObj, Resolve,
};

/// In Type 1 (function-based) shadings, the colour at every point in the domain is defined by a specified
/// mathematical function. The function need not be smooth or continuous. This type is the most general of the
/// available shading types and is useful for shadings that cannot be adequately described with any of the other
/// types
#[derive(Debug, Clone, FromObj)]
pub struct FunctionBasedShading<'a> {
    /// An array of four numbers [xmin xmax ymin ymax] specifying the rectangular domain of coordinates
    /// over which the colour function(s) are defined
    ///
    /// Default value: [0.0 1.0 0.0 1.0].
    #[field("Domain", default = FunctionDomain::default())]
    domain: FunctionDomain,

    /// An array of six numbers specifying a transformation matrix mapping the coordinate space specified
    /// by the Domain entry into the shading's target coordinate space
    ///
    /// EXAMPLE To map the domain rectangle [0.0 1.0 0.0 1.0] to a 1-inch square with lower-left corner at
    /// coordinates (100, 100) in default user space, the Matrix value would be [72 0 0 72 100 100]
    #[field("Matrix")]
    matrix: Option<Matrix>,

    /// A 2-in, n-out function or an array of n 2-in, 1-out functions (where n is the number of
    /// colour components in the shading dictionary's colour space). Each function's domain shall
    /// be a superset of that of the shading dictionary. If the value returned by the function for
    /// a given colour component is out of range, it shall be adjusted to the nearest valid value
    #[field("Function")]
    function: Function<'a>,
}

#[derive(Debug, Clone, Copy)]
struct FunctionDomain {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl<'a> FromObj<'a> for FunctionDomain {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut arr = resolver.assert_arr(obj)?;
        assert_len(&arr, 4)?;

        let y_max = resolver.assert_number(arr.pop().unwrap())?;
        let y_min = resolver.assert_number(arr.pop().unwrap())?;
        let x_max = resolver.assert_number(arr.pop().unwrap())?;
        let x_min = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Self {
            x_min,
            x_max,
            y_min,
            y_max,
        })
    }
}

impl Default for FunctionDomain {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        }
    }
}
