use crate::data_structures::Matrix;

use super::{BoundingBox, CubicBezierCurve, Line, Point};

#[derive(Debug, Clone, Copy)]
pub struct QuadraticBezierCurve {
    pub start: Point,
    pub end: Point,
    pub control_point: Point,
}

#[derive(Debug, Clone, Copy)]
struct Parabola {
    x0: f32,
    x2: f32,
    scale: f32,
    cross: f32,
}

impl QuadraticBezierCurve {
    pub fn new(start: Point, end: Point, control_point: Point) -> Self {
        Self {
            start,
            control_point,
            end,
        }
    }

    fn into_cubic(self) -> CubicBezierCurve {
        let q0 = self.start;
        let q1 = (1.0 / 3.0) * self.start + (2.0 / 3.0) * self.control_point;
        let q2 = (2.0 / 3.0) * self.control_point + (1.0 / 3.0) * self.end;
        let q3 = self.end;

        CubicBezierCurve::new(q0, q3, q1, q2)
    }

    pub fn basis(&self, t: f32) -> Point {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        self.start * mt2 + self.control_point * 2.0 * mt * t + self.end * t2
    }

    /// Algorithm adapted from <https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html>
    pub fn subdivide(&self, tolerance: f32) -> Vec<Line> {
        /// Compute an approximation to int (1 + 4x^2) ^ -0.25 dx
        fn approx_myint(x: f32) -> f32 {
            const D: f32 = 0.67;

            x / (1.0 - D + (D.powf(4.0) + 0.25 * x * x).powf(0.25))
        }

        /// Approximate the inverse of the function above
        fn approx_inv_myint(x: f32) -> f32 {
            const B: f32 = 0.39;

            x * (1.0 - B + (B * B + 0.25 * x * x).sqrt())
        }

        let params = self.map_to_basic();

        let a0 = approx_myint(params.x0);
        let a2 = approx_myint(params.x2);

        let count = 0.5 * (a2 - a0).abs() * (params.scale / tolerance).sqrt();

        let n = count.ceil();

        let x0 = approx_inv_myint(a0);
        let x2 = approx_inv_myint(a2);

        let mut result = vec![0.0];

        for i in 0..(n as i32) {
            let x = approx_inv_myint(a0 + ((a2 - a0) * i as f32) / n);

            let t = (x - x0) / (x2 - x0);

            result.push(t);
        }

        result.push(1.0);

        result
            .windows(2)
            .map(|arr| {
                let start = self.basis(arr[0]);
                let end = self.basis(arr[1]);

                Line::new(start, end)
            })
            .collect()
    }

    fn map_to_basic(&self) -> Parabola {
        let dd = 2.0 * self.control_point - self.start - self.end;

        let u0 = (self.control_point.x - self.start.x) * dd.x
            + (self.control_point.y - self.start.y) * dd.y;
        let u2 =
            (self.end.x - self.control_point.x) * dd.x + (self.end.y - self.control_point.y) * dd.y;

        let cross = (self.end.x - self.start.x) * dd.y - (self.end.y - self.start.y) * dd.x;

        let x0 = u0 / cross;
        let x2 = u2 / cross;

        let scale = cross.abs() / dd.x.hypot(dd.y) * (x2 - x0).abs();

        Parabola {
            x0,
            x2,
            scale,
            cross,
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        // todo: suboptimal
        self.into_cubic().bounding_box()
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        self.start *= transformation;
        self.end *= transformation;
        self.control_point *= transformation;
    }
}
