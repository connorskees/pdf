use crate::data_structures::Matrix;

use super::{BoundingBox, CubicBezierCurve, Line, Point, QuadraticBezierCurve};

#[derive(Debug, Clone, Copy)]
pub enum Subpath {
    Line(Line),
    Quadratic(QuadraticBezierCurve),
    Cubic(CubicBezierCurve),
}

impl Subpath {
    pub fn apply_transform(&mut self, transformation: Matrix) {
        match self {
            Self::Line(line) => line.apply_transform(transformation),
            Self::Quadratic(curve) => curve.apply_transform(transformation),
            Self::Cubic(curve) => curve.apply_transform(transformation),
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        match self {
            Self::Line(line) => line.bounding_box(),
            Self::Quadratic(curve) => curve.bounding_box(),
            Self::Cubic(curve) => curve.bounding_box(),
        }
    }

    /// Flatten a subpath into a series of Lines
    pub fn flatten(self) -> Vec<Line> {
        match self {
            Subpath::Line(line) => vec![line],
            Subpath::Quadratic(curve) => curve.subdivide(0.01),
            Subpath::Cubic(curve) => curve.approximate_quadratic().subdivide(0.01),
        }
    }

    pub fn is_line(self) -> bool {
        matches!(self, Subpath::Line(..))
    }

    pub fn start(&self) -> Point {
        match self {
            Subpath::Line(line) => line.start,
            Subpath::Quadratic(curve) => curve.start,
            Subpath::Cubic(curve) => curve.start,
        }
    }

    pub fn end(&self) -> Point {
        match self {
            Subpath::Line(line) => line.end,
            Subpath::Quadratic(curve) => curve.end,
            Subpath::Cubic(curve) => curve.end,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub subpaths: Vec<Subpath>,
    pub current_point: Point,
    pub start: Point,
}

impl Path {
    pub const fn new(start: Point) -> Self {
        Self {
            subpaths: Vec::new(),
            current_point: start,
            start,
        }
    }

    pub const fn from_subpaths(subpaths: Vec<Subpath>) -> Self {
        Self {
            subpaths,
            current_point: Point::new(0.0, 0.0),
            start: Point::new(0.0, 0.0),
        }
    }

    // todo: implement clipping
    // maybe see:
    //   * https://davis.wpi.edu/~matt/courses/clipping/
    //   * https://en.wikipedia.org/wiki/Sutherland%E2%80%93Hodgman_algorithm
    //   * https://en.wikipedia.org/wiki/Weiler%E2%80%93Atherton_clipping_algorithm
    //   * https://en.wikipedia.org/wiki/Vatti_clipping_algorithm
    //   * Martinez-Rueda (https://github.com/w8r/martinez)
    //   * https://en.wikipedia.org/wiki/Greiner%E2%80%93Hormann_clipping_algorithm
    pub fn clip(&mut self, _clipping_path: &Path) {}

    pub fn close_path(&mut self) {
        if self.start != self.current_point {
            self.line_to(self.start);
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::new();

        for subpath in &self.subpaths {
            bbox.merge(subpath.bounding_box());
        }

        bbox
    }

    pub fn stroke_for_line(line: Line, width: f32) -> Path {
        let n = (line.end - line.start)
            .rotate_90()
            .with_distance_from_origin(width / 2.0);
        let mut p = Path::new(line.start + n);
        p.line_to(line.end + n);
        p.line_to(line.end - n);
        p.line_to(line.start - n);
        p.close_path();
        p
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

    /// Flatten bezier curves in subpaths into a series of lines
    pub fn flatten(&mut self) {
        self.subpaths = std::mem::take(&mut self.subpaths)
            .into_iter()
            .flat_map(|subpath| subpath.flatten())
            .map(Subpath::Line)
            .collect();
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
                Subpath::Quadratic(curve) => {
                    for line2 in curve.subdivide(0.01) {
                        if line2.intersects_line(line) {
                            count += 1;
                        }
                    }
                }
                Subpath::Cubic(curve) => {
                    for line2 in curve.approximate_quadratic().subdivide(0.01) {
                        if line2.intersects_line(line) {
                            count += 1;
                        }
                    }
                }
            }
        }

        (count & 1) != 0
    }

    pub fn quadratic_curve_to(&mut self, control_point: Point, end: Point) {
        self.subpaths
            .push(Subpath::Quadratic(QuadraticBezierCurve::new(
                self.current_point,
                end,
                control_point,
            )));
        self.current_point = end;
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

    pub fn to_js_path(&self) -> String {
        let mut out = String::new();

        out += "ctx.beginPath();\n";
        out += &format!(
            "ctx.moveTo({}, {});\n",
            self.subpaths[0].start().x,
            self.subpaths[0].start().y
        );
        let mut last_point = self.subpaths[0].start();
        for subpath in &self.subpaths {
            if subpath.start() != last_point {
                out += &format!(
                    "ctx.moveTo({}, {});\n",
                    subpath.start().x,
                    subpath.start().y
                );
            }
            match subpath {
                Subpath::Line(l) => out += &format!("ctx.lineTo({}, {});\n", l.end.x, l.end.y),
                Subpath::Quadratic(quad) => {
                    out += &format!(
                        "ctx.quadraticCurveTo({}, {}, {}, {});\n",
                        quad.control_point.x, quad.control_point.y, quad.end.x, quad.end.y
                    );
                }
                Subpath::Cubic(cub) => {
                    out += &format!(
                        "ctx.bezierCurveTo({}, {}, {}, {}, {}, {});\n",
                        cub.first_control_point.x,
                        cub.first_control_point.y,
                        cub.second_control_point.x,
                        cub.second_control_point.y,
                        cub.end.x,
                        cub.end.y
                    );
                }
            }

            last_point = subpath.end();
        }
        out += "ctx.closePath();\n";
        out += "ctx.fill();\n";

        out
    }
}
