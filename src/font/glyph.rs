use crate::geometry::{Outline, Point};

pub struct Glyph {
    pub(crate) outline: Outline,
    pub(crate) width_vector: Point,
}
