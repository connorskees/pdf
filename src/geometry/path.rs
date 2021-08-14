use crate::data_structures::Matrix;

use super::{BoundingBox, CubicBezierCurve, Line, Point};

#[derive(Debug, Clone, Copy)]
pub enum Subpath {
    Line(Line),
    Cubic(CubicBezierCurve),
}

impl Subpath {
    pub fn apply_transform(&mut self, transformation: Matrix) {
        match self {
            Self::Line(line) => line.apply_transform(transformation),
            Self::Cubic(curve) => curve.apply_transform(transformation),
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        match self {
            Self::Line(line) => line.bounding_box(),
            Self::Cubic(curve) => curve.bounding_box(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub subpaths: Vec<Subpath>,
    pub current_point: Point,
    start: Point,
}

impl Path {
    pub const fn new(start: Point) -> Self {
        Self {
            subpaths: Vec::new(),
            current_point: start,
            start,
        }
    }

    pub fn close_path(&mut self) {
        self.line_to(self.start);
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::new();

        for subpath in &self.subpaths {
            bbox.merge(subpath.bounding_box());
        }

        bbox
    }

    pub fn apply_transform(&mut self, transformation: Matrix) {
        for subpath in &mut self.subpaths {
            subpath.apply_transform(transformation);
        }
    }

    pub fn move_to(&mut self, point: Point) {
        self.current_point = point;

        if self.subpaths.is_empty() {
            self.start = self.current_point;
        }
    }

    pub fn relative_move_to(&mut self, dx: f32, dy: f32) {
        let point = Point::new(self.current_point.x + dx, self.current_point.y + dy);

        self.move_to(point);
    }

    pub fn line_to(&mut self, p: Point) {
        self.subpaths
            .push(Subpath::Line(Line::new(self.current_point, p)));
        self.current_point = p;
    }

    pub fn relative_line_to(&mut self, dx: f32, dy: f32) {
        let end = Point::new(self.current_point.x + dx, self.current_point.y + dy);

        self.line_to(end);
    }

    pub fn intersects_line_even_odd(&self, line: Line) -> bool {
        let mut count = 0;
        for path in &self.subpaths {
            match path {
                Subpath::Line(line2) => {
                    if line2.intersects_line(line) {
                        count += 1;
                    }
                }
                Subpath::Cubic(..) => todo!(),
            }
        }

        (count & 1) != 0
    }

    pub fn cubic_curve_to(
        &mut self,
        first_control_point: Point,
        second_control_point: Point,
        end: Point,
    ) {
        self.subpaths.push(Subpath::Cubic(CubicBezierCurve::new(
            self.current_point,
            end,
            first_control_point,
            second_control_point,
        )));
        self.current_point = end;
    }
}
