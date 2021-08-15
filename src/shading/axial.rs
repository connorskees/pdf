use std::convert::TryInto;

use crate::{
    catalog::assert_len,
    error::{ParseError, PdfResult},
    function::Function,
    objects::{Dictionary, Object},
    Resolve,
};

/// Type 2 (axial) shadings define a colour blend that varies along a linear axis between two
/// endpoints and extends indefinitely perpendicular to that axis. The shading may optionally
/// be extended beyond either or both endpoints by continuing the boundary colours indefinitely
///
/// This type of shading shall not be used with an Indexed colour space.
#[derive(Debug, Clone)]
pub struct AxialShading<'a> {
    /// An array of four numbers [x0 y0 x1 y1] specifying the starting and ending coordinates
    /// of the axis, expressed in the shading's target coordinate space
    coords: Coords,

    /// An array of two numbers [t0 t1] specifying the limiting values of a parametric variable
    /// t. The variable is considered to vary linearly between these two values as the colour
    /// gradient varies between the starting and ending points of the axis. The variable t becomes
    /// the input argument to the colour function(s)
    ///
    /// Default value: [0.0 1.0].
    domain: [f32; 2],

    /// A 1-in, n-out function or an array of n 1-in, 1-out functions (where n is the number of
    /// colour components in the shading dictionary's colour space). The function(s) shall be
    /// called with values of the parametric variable t in the domain defined by the Domain entry.
    /// Each function's domain shall be a superset of that of the shading dictionary. If the value
    /// returned by the function for a given colour component is out of range, it shall be adjusted
    /// to the nearest valid value
    function: Function<'a>,

    /// An array of two boolean values specifying whether to extend the shading beyond the starting
    /// and ending points of the axis, respectively. Default value: [false false].
    extend: [bool; 2],
}

impl<'a> AxialShading<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let coords = Coords::from_arr(dict.expect_arr("Coords", resolver)?, resolver)?;
        let domain = dict
            .get_arr("Domain", resolver)?
            .map(|arr| -> Result<_, ParseError> {
                assert_len(&arr, 2)?;

                Ok(arr
                    .into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()?
                    .try_into()
                    .unwrap())
            })
            .transpose()?
            .unwrap_or([0.0, 1.0]);
        let function = dict.expect_function("Function", resolver)?;
        let extend = dict
            .get_arr("Extend", resolver)?
            .map(|arr| -> Result<_, ParseError> {
                assert_len(&arr, 2)?;

                Ok(arr
                    .into_iter()
                    .map(|obj| resolver.assert_bool(obj))
                    .collect::<PdfResult<Vec<bool>>>()?
                    .try_into()
                    .unwrap())
            })
            .transpose()?
            .unwrap_or([false, false]);

        Ok(Self {
            coords,
            domain,
            function,
            extend,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Coords {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl Coords {
    pub fn from_arr<'a>(mut arr: Vec<Object>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        assert_len(&arr, 4)?;

        let y1 = resolver.assert_number(arr.pop().unwrap())?;
        let x1 = resolver.assert_number(arr.pop().unwrap())?;
        let y0 = resolver.assert_number(arr.pop().unwrap())?;
        let x0 = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Self { x0, y0, x1, y1 })
    }
}
