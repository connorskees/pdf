use crate::data_structures::Matrix;

use super::{point::Point, BoundingBox, QuadraticBezierCurve};

pub fn solve_quadratic_formula(a: f32, b: f32, c: f32) -> [f32; 2] {
    let sqrt = ((b * b) - (4.0 * a * c)).sqrt();
    let a2 = 2.0 * a;

    [(-b + sqrt) / a2, (-b - sqrt) / a2]
}

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

        self.start * mt3
            + 3.0 * self.first_control_point * mt2 * t
            + 3.0 * self.second_control_point * mt * t2
            + self.end * t3
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        self.start *= transformation;
        self.end *= transformation;
        self.first_control_point *= transformation;
        self.second_control_point *= transformation;
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::new();

        let a = -3.0 * self.start + 9.0 * self.first_control_point
            - 9.0 * self.second_control_point
            + 3.0 * self.end;
        let b =
            6.0 * self.start - 12.0 * self.first_control_point + 6.0 * self.second_control_point;
        let c = -3.0 * self.start + 3.0 * self.first_control_point;

        let t_x = solve_quadratic_formula(a.x, b.x, c.x);
        let t_y = solve_quadratic_formula(a.y, b.y, c.y);

        for t in [t_x[0], t_x[1], t_y[0], t_y[1]] {
            if t <= 1.0 && t >= 0.0 {
                bbox.add_point(self.basis(t));
            }
        }

        bbox.add_point(self.start);
        bbox.add_point(self.end);

        bbox
    }

    pub fn approximate_quadratic(self) -> QuadraticBezierCurve {
        let first_p1 = (3.0 / 2.0) * self.first_control_point - 0.5 * self.start;
        let second_p1 = (3.0 / 2.0) * self.second_control_point - 0.5 * self.end;

        let p1 = (first_p1 + second_p1) / 2.0;

        QuadraticBezierCurve {
            start: self.start,
            control_point: p1,
            end: self.end,
        }
    }
}
