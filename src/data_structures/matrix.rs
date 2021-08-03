use std::ops::{Mul, MulAssign};

use crate::{catalog::assert_len, error::PdfResult, objects::Object, Resolve};

/// A 3x3 matrix
///
/// It is only possible to specify 6 out of the 9 possible values.
///
/// The full matrix is of the form:
///
/// [a b 0]
/// [c d 0]
/// [e f 1]
#[derive(Debug, Clone, Copy)]
pub struct Matrix {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;
    fn mul(self, other: Matrix) -> Self::Output {
        let a = self.a * other.a + self.b * other.c + 0.0 * other.e;
        let c = self.c * other.a + self.d * other.c + 0.0 * other.e;
        let e = self.e * other.a + self.f * other.c + 1.0 * other.e;

        let b = self.a * other.b + self.b * other.d + 0.0 * other.f;
        let d = self.c * other.b + self.d * other.d + 0.0 * other.f;
        let f = self.e * other.b + self.f * other.d + 1.0 * other.f;

        Matrix::new(a, b, c, d, e, f)
    }
}

impl MulAssign<Matrix> for Matrix {
    fn mul_assign(&mut self, rhs: Matrix) {
        *self = *self * rhs;
    }
}

impl Matrix {
    pub fn identity() -> Self {
        Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn new_transform(x: f32, y: f32) -> Self {
        let mut identity = Self::identity();

        identity.e = x;
        identity.f = y;

        identity
    }

    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 6)?;

        let f = resolver.assert_number(arr.pop().unwrap())?;
        let e = resolver.assert_number(arr.pop().unwrap())?;
        let d = resolver.assert_number(arr.pop().unwrap())?;
        let c = resolver.assert_number(arr.pop().unwrap())?;
        let b = resolver.assert_number(arr.pop().unwrap())?;
        let a = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Self { a, b, c, d, e, f })
    }
}
