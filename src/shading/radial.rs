use std::convert::TryInto;

use crate::{
    catalog::assert_len,
    error::PdfResult,
    function::Function,
    objects::{Dictionary, Object},
    Resolve,
};

/// Type 3 (radial) shadings define a colour blend that varies between two circles. Shadings
/// of this type are commonly used to depict three-dimensional spheres and cones
///
/// This type of shading shall not be used with an Indexed colour space
#[derive(Debug, Clone)]
pub struct RadialShading<'a> {
    /// An array of six numbers [x0 y0 r0 x1 y1 r1] specifying the centres and radii of
    /// the starting and ending circles, expressed in the shading's target coordinate
    /// space. The radii r0 and r1 shall both be greater than or equal to 0. If one
    /// radius is 0, the corresponding circle shall be treated as a point; if both are
    /// 0, nothing shall be painted
    coords: Coords,

    /// An array of two numbers [t0 t1] specifying the limiting values of a parametric
    /// variable t. The variable is considered to vary linearly between these two values
    /// as the colour gradient varies between the starting and ending circles. The variable
    /// t becomes the input argument to the colour function(s)
    ///
    /// Default value: [0.0 1.0].
    domain: [f32; 2],

    /// A 1-in, n-out function or an array of n 1-in, 1-out functions (where n is the
    /// number of colour components in the shading dictionary's colour space). The
    /// function(s) shall be called with values of the parametric variable t in the domain
    /// defined by the shading dictionary's Domain entry. Each function's domain shall be
    /// a superset of that of the shading dictionary. If the value returned by the function
    /// for a given colour component is out of range, it shall be adjusted to the nearest
    /// valid value.
    function: Function<'a>,

    /// An array of two boolean values specifying whether to extend the shading beyond the
    /// starting and ending circles, respectively
    ///
    /// Default value: [false false].
    extend: [bool; 2],
}

impl<'a> RadialShading<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let coords = Coords::from_arr(dict.expect_arr("Coords", resolver)?, resolver)?;
        let domain = dict
            .get_arr("Domain", resolver)?
            .map(|arr| -> PdfResult<[f32; 2]> {
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
            .get_arr("Domain", resolver)?
            .map(|arr| -> PdfResult<[bool; 2]> {
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

#[derive(Debug, Clone)]
struct Coords {
    start: Circle,
    end: Circle,
}

impl Coords {
    pub fn from_arr<'a>(mut arr: Vec<Object>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        assert_len(&arr, 6)?;

        let end = Circle::from_arr(arr.split_off(3), resolver)?;
        let start = Circle::from_arr(arr, resolver)?;

        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy)]
struct Circle {
    x: f32,
    y: f32,
    radius: f32,
}

impl Circle {
    pub fn from_arr<'a>(mut arr: Vec<Object>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        assert_len(&arr, 3)?;

        let radius = resolver.assert_number(arr.pop().unwrap())?;
        let y = resolver.assert_number(arr.pop().unwrap())?;
        let x = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Self { x, y, radius })
    }
}
