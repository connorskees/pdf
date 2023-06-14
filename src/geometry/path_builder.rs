use super::{Outline, Path, Point};

#[derive(Debug)]
pub struct PathBuilder {
    pub outline: Outline,
    pub width_vector: Point,
    pub current_path: Path,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            current_path: Path::new(Point::new(0.0, 0.0)),
            outline: Outline::empty(),
            width_vector: Point::new(0.0, 0.0),
        }
    }

    pub fn relative_line_to(&mut self, dx: f32, dy: f32) {
        self.current_path.relative_line_to(dx, dy);
    }

    pub fn horizontal_vertical_curve_to(&mut self, dx1: f32, dx2: f32, dy2: f32, dy3: f32) {
        self.relative_relative_curve_to(dx1, 0.0, dx2, dy2, 0.0, dy3)
    }

    pub fn vertical_horizontal_curve_to(&mut self, dy1: f32, dx2: f32, dy2: f32, dx3: f32) {
        self.relative_relative_curve_to(0.0, dy1, dx2, dy2, dx3, 0.0)
    }

    pub fn vertical_line_to(&mut self, dy: f32) {
        self.current_path.relative_line_to(0.0, dy);
    }

    pub fn close_path(&mut self) {
        let current_point = self.current_path.current_point;
        self.current_path.close_path();

        self.outline.paths.push(self.current_path.clone());
        self.current_path = Path::new(current_point);
    }

    pub fn relative_move_to(&mut self, dx: f32, dy: f32) {
        self.current_path.relative_move_to(dx, dy);
    }

    pub fn relative_relative_curve_to(
        &mut self,
        dx1: f32,
        dy1: f32,
        dx2: f32,
        dy2: f32,
        dx3: f32,
        dy3: f32,
    ) {
        let current_point = self.current_path.current_point;

        let first_control_point = Point::new(current_point.x + dx1, current_point.y + dy1);
        let second_control_point =
            Point::new(current_point.x + dx1 + dx2, current_point.y + dy1 + dy2);
        let end = Point::new(
            current_point.x + dx1 + dx2 + dx3,
            current_point.y + dy1 + dy2 + dy3,
        );

        self.current_path
            .cubic_curve_to(first_control_point, second_control_point, end);
    }

    pub fn horizontal_line_to(&mut self, dx: f32) {
        self.current_path.relative_line_to(dx, 0.0);
    }

    pub fn hsbw(&mut self, side_bearing_x_coord: f32, width_vector_x_coord: f32) {
        self.current_path = Path::new(Point::new(side_bearing_x_coord, 0.0));
        self.width_vector = Point::new(width_vector_x_coord, 0.0);
    }

    #[allow(unused)]
    pub fn horizontal_stem(&mut self, y: f32, dy: f32) {}
    #[allow(unused)]
    pub fn vertical_stem(&mut self, x: f32, dx: f32) {}
    #[allow(unused)]
    pub fn horizontal_stem3(&mut self, y0: f32, dy0: f32, y1: f32, dy1: f32, y2: f32, dy2: f32) {}
    #[allow(unused)]
    pub fn vertical_stem3(&mut self, x0: f32, dx0: f32, x1: f32, dx1: f32, x2: f32, dx2: f32) {}
}
