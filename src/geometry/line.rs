use crate::data_structures::Matrix;

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
}
