use crate::{catalog::assert_len, error::PdfResult, objects::Object, Resolve};

#[derive(Debug)]
pub struct Matrix {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
}

impl Matrix {
    pub fn identity() -> Self {
        Matrix {
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
        }
    }

    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 6)?;

        let b2 = resolver.assert_number(arr.pop().unwrap())?;
        let b1 = resolver.assert_number(arr.pop().unwrap())?;
        let b0 = resolver.assert_number(arr.pop().unwrap())?;
        let a2 = resolver.assert_number(arr.pop().unwrap())?;
        let a1 = resolver.assert_number(arr.pop().unwrap())?;
        let a0 = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Self {
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
        })
    }
}
