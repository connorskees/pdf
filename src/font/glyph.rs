use crate::geometry::{Outline, Point};

#[derive(Debug, Clone)]
pub struct Glyph {
    pub(crate) outline: Outline,
    pub(crate) width_vector: Point,
}

impl Glyph {
    pub const fn empty() -> Self {
        Self {
            outline: Outline::empty(),
            width_vector: Point::new(0.0, 0.0),
        }
    }
}
