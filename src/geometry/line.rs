use std::ops::Mul;

use crate::{data_structures::Matrix, render::canvas::fuzzy_eq};

use super::{point::Point, BoundingBox};

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    pub fn slope(&self) -> f32 {
        (self.end.y - self.start.y) / (self.end.x - self.start.x)
    }

    pub fn rotate_90(&self) -> Self {
        Self::new(self.start.rotate_90(), self.end.rotate_90())
    }

    pub fn length(&self) -> f32 {
        self.start.euclidean_distance(self.end)
    }

    pub fn with_length(&self, new_len: f32) -> Line {
        let len = self.length();
        *self * (new_len / len)
    }

    pub fn y_intercept(&self) -> f32 {
        let slope = self.slope();

        self.start.y - slope * self.start.x
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        self.start *= transformation;
        self.end *= transformation;
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::new();

        bbox.add_point(self.start);
        bbox.add_point(self.end);

        bbox
    }

    pub fn is_left_to_right(&self) -> bool {
        self.start.x < self.end.x
    }

    pub fn intersects_line(&self, other: Line) -> bool {
        let f = |x: f32, y: f32| -> f32 {
            (x - self.start.x) * (self.end.y - self.start.y)
                - (y - self.start.y) * (self.end.x - self.start.x)
        };

        let g = |x: f32, y: f32| -> f32 {
            (x - other.start.x) * (other.end.y - other.start.y)
                - (y - other.start.y) * (other.end.x - other.start.x)
        };

        f(other.start.x, other.start.y) * f(other.end.x, other.end.y) < 0.0
            && g(self.start.x, self.start.y) * g(self.end.x, self.end.y) < 0.0
    }

    pub fn is_horizontal(&self) -> bool {
        fuzzy_eq(self.slope(), 0.0)
    }

    pub fn x_axis() -> Self {
        Self::new(
            Point::new(f32::NEG_INFINITY, 0.0),
            Point::new(f32::INFINITY, 0.0),
        )
    }
}

impl Mul<f32> for Line {
    type Output = Line;

    fn mul(self, rhs: f32) -> Self::Output {
        Line {
            start: self.start * rhs,
            end: self.end * rhs,
        }
    }
}

impl Mul<Line> for f32 {
    type Output = Line;

    fn mul(self, rhs: Line) -> Self::Output {
        Line {
            start: self * rhs.start,
            end: self * rhs.end,
        }
    }
}
