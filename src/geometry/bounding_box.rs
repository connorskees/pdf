use super::Point;
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point {
                x: f32::INFINITY,
                y: f32::INFINITY,
            },
            max: Point {
                x: f32::NEG_INFINITY,
                y: f32::NEG_INFINITY,
            },
        }
    }

    pub fn add_point(&mut self, p: Point) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn contains_point(&self, p: Point) -> bool {
        p.x > self.min.x && p.y > self.min.y && p.x < self.max.x && p.y < self.max.y
    }

    pub fn merge(&mut self, other: Self) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);

        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}
