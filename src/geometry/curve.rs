use crate::{data_structures::Matrix, render::canvas::fuzzy_eq};

use super::{point::Point, BoundingBox, Line};

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

            t += 0.001;
        }

        bbox
    }

    pub fn find_number_of_intersections(&self, line: Line) -> usize {
        let a = -self.start + 3.0 * self.first_control_point - 3.0 * self.second_control_point
            + self.end;
        let b = 3.0 * self.start - 6.0 * self.first_control_point + 3.0 * self.second_control_point;
        let c = -3.0 * self.start + 3.0 * self.first_control_point;
        let d = self.start;

        let t = Self::solve(a.y, b.y, c.y, d.y - line.start.y);

        let x = |t: f32| -> f32 { a.x * t.powf(3.0) + b.x * t.powf(2.0) + c.x * t + d.x };

        t.into_iter()
            .filter(|&t| {
                assert!(t.is_finite());

                if t > 1.0 || t < 0.0 {
                    return false;
                }

                assert!(t >= 0.0 && t < 1.0, "{}", t);

                let x = x(t);

                x > line.start.x && x < line.end.x
            })
            .count()
    }

    fn solve(a: f32, b: f32, c: f32, d: f32) -> Vec<f32> {
        let p = ((3.0 * a * c) - b.powf(2.0)) / (3.0 * a.powf(2.0));
        let q = ((2.0 * b.powf(2.0)) - (9.0 * a * b * c) + (27.0 * a.powf(2.0) * d))
            / (27.0 * a.powf(3.0));

        Self::solve_depressed(p, q)
    }

    fn solve_depressed(p: f32, q: f32) -> Vec<f32> {
        let d = (q.powf(2.0) / 4.0) + (p.powf(3.0) / 27.0);

        if fuzzy_eq(p, 0.0) && fuzzy_eq(q, 0.0) {
            vec![0.0]
        } else if fuzzy_eq(d, 0.0) && !fuzzy_eq(p, 0.0) {
            vec![(3.0 * q) / p, -((3.0 * q) / (2.0 * p))]
        } else if d < 0.0 && !fuzzy_eq(p, 0.0) {
            Self::solve_trig(p, q).to_vec()
        } else {
            vec![Self::solve_cardano(d)]
        }
    }

    fn solve_trig(p: f32, q: f32) -> [f32; 3] {
        let u = (1.0 / 3.0) * (((3.0 * q) / (2.0 * p)) * (-(3.0 / p)).sqrt()).acos();
        let v = 2.0 * (-(1.0 / 3.0) * p).sqrt();

        [
            v * u.cos(),
            v * (u - ((2.0 / 3.0) * std::f32::consts::PI)).cos(),
            v * (u - ((4.0 / 3.0) * std::f32::consts::PI)).cos(),
        ]
    }

    fn solve_cardano(d: f32) -> f32 {
        let neg_half = -0.5;
        let sqrt = d.sqrt();

        (neg_half + sqrt).cbrt() + (neg_half - sqrt).cbrt()
    }
}
