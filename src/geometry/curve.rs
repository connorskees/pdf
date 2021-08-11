use crate::data_structures::Matrix;

use super::{point::Point, BoundingBox};

#[derive(Debug, Clone, Copy)]
pub struct CubicBezierCurve {
    pub start: Point,
    pub end: Point,
    pub first_control_point: Point,
    pub second_control_point: Point,
}

impl CubicBezierCurve {
    pub fn new(
        start: Point,
        end: Point,
        first_control_point: Point,
        second_control_point: Point,
    ) -> Self {
        Self {
            start,
            first_control_point,
            second_control_point,
            end,
        }
    }

    pub fn basis(&self, t: f32) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;

        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let x = self.start.x * mt3
            + 3.0 * self.first_control_point.x * mt2 * t
            + 3.0 * self.second_control_point.x * mt * t2
            + self.end.x * t3;

        let y = self.start.y * mt3
            + 3.0 * self.first_control_point.y * mt2 * t
            + 3.0 * self.second_control_point.y * mt * t2
            + self.end.y * t3;

        Point::new(x, y)
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        self.start *= transformation;
        self.end *= transformation;
        self.first_control_point *= transformation;
        self.second_control_point *= transformation;
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut t = 0.0;

        let mut bbox = BoundingBox::new();

        while t < 1.0 {
            let p = self.basis(t);

            bbox.add_point(p);

            t += 0.0001;
        }

        bbox
    }
}
