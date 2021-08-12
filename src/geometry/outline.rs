use crate::data_structures::Matrix;

use super::{BoundingBox, Path};

#[derive(Debug, Clone)]
pub struct Outline {
    pub paths: Vec<Path>,
}

impl Outline {
    pub const fn empty() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        for path in &mut self.paths {
            path.apply_transform(transformation);
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::new();

        for path in &self.paths {
            bbox.merge(path.bounding_box());
        }

        bbox
    }
}
