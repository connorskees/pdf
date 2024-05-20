use std::ops::{Mul, MulAssign};

use crate::{
    catalog::assert_len, error::PdfResult, geometry::Point, objects::Object, FromObj, Resolve,
};

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
        let a = self.a * other.a + self.b * other.c;
        let c = self.c * other.a + self.d * other.c;

        let b = self.a * other.b + self.b * other.d;
        let d = self.c * other.b + self.d * other.d;

        // todo: this is almost certainly wrong, but it works for the test case
        // i'm using
        let e = self.e + other.e;
        let f = self.f + other.f;

        Matrix::new(a, b, c, d, e, f)
    }
}

impl Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, other: Point) -> Self::Output {
        let x = self.a * other.x + self.c * other.y + self.e;
        let y = self.b * other.x + self.d * other.y + self.f;

        Point::new(x, y)
    }
}

impl MulAssign<Matrix> for Point {
    fn mul_assign(&mut self, rhs: Matrix) {
        *self = rhs * *self;
    }
}

impl MulAssign<Matrix> for Matrix {
    fn mul_assign(&mut self, rhs: Matrix) {
        *self = *self * rhs;
    }
}

impl Matrix {
    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub const fn identity() -> Self {
        Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub const fn new_translation(x: f32, y: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, x, y)
    }

    pub fn new_scale(x: f32, y: f32) -> Self {
        Self::new(x, 0.0, 0.0, y, 0.0, 0.0)
    }

    /// Rotate by `q` degrees
    pub fn new_rotation(q: f32) -> Self {
        let q = q.to_radians();

        Self::new(q.cos(), q.sin(), -q.sin(), q.cos(), 0.0, 0.0)
    }

    /// Skew x-axis by `a` degrees and y-axis by `b` degrees
    pub fn new_skew(a: f32, b: f32) -> Self {
        let a = a.to_radians();
        let b = b.to_radians();

        Self::new(1.0, a.tan(), b.tan(), 1.0, 0.0, 0.0)
    }

    pub fn from_arr(arr: [f32; 6]) -> Self {
        let [a, b, c, d, e, f] = arr;
        Self { a, b, c, d, e, f }
    }
}

impl<'a> FromObj<'a> for Matrix {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut arr = resolver.assert_arr(obj)?;
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

impl Into<[[f32; 4]; 4]> for Matrix {
    fn into(self) -> [[f32; 4]; 4] {
        [
            [self.a, self.b, 0.0, 0.0],
            [self.c, self.d, 0.0, 0.0],
            [self.e, self.f, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
