use crate::{
    error::PdfResult,
    filter::decode_stream,
    geometry::{CubicBezierCurve, Line, Outline, Path, Point, Subpath},
    resolve::Resolve,
    xobject::ImageXObject,
};

use minifb::{Key, Window, WindowOptions};

pub fn fuzzy_eq(a: f32, b: f32) -> bool {
    let a = a.abs();
    let b = b.abs();
    let diff = (a - b).abs();

    if a == b {
        true
    } else if a == 0.0 || b == 0.0 || diff < f32::MIN {
        diff < (f32::EPSILON * f32::MIN)
    } else {
        diff / (a + b) < f32::EPSILON
    }
}

pub(super) struct Canvas {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    window: Window,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let mut window = Window::new("PDF", width, height, WindowOptions::default()).unwrap();

        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Self {
            width,
            height,
            buffer: vec![u32::MAX; width * height],
            window,
        }
    }

    pub fn fill_path_non_zero_winding_number(&mut self, _path: &Path, _color: u32) {
        todo!()
    }

    pub fn fill_path_even_odd(&mut self, path: &Path, color: u32) {
        let bbox = path.bounding_box();

        let x = bbox.min.x as u32;
        let y = bbox.min.y as u32;

        for h in y..(bbox.height().ceil() as u32 + y) {
            for w in x..(bbox.width().ceil() as u32 + x) {
                let point = Point::new(w as f32, h as f32);

                if path.intersects_line_even_odd(Line::new(
                    point,
                    Point::new(bbox.max.x + 1.0, bbox.max.y + 1.0),
                )) {
                    self.paint_point(point, color);
                }
            }
        }
    }

    pub fn stroke_outline(&mut self, outline: &Outline, color: u32) {
        for path in &outline.paths {
            self.stroke_path(path, color);
        }
    }

    pub fn stroke_path(&mut self, path: &Path, color: u32) {
        for &subpath in &path.subpaths {
            match subpath {
                Subpath::Line(line) => self.draw_line(line, color),
                Subpath::Cubic(curve) => self.draw_cubic_bezier_curve(curve, color),
            }
        }
    }

    pub fn draw_line(&mut self, line: Line, color: u32) {
        let mut start = (line.start.x as i32, line.start.y as i32);
        let end = (line.end.x as i32, line.end.y as i32);

        let dx = (end.0 - start.0).abs() as i32;
        let dy = -(end.1 - start.1).abs() as i32;

        let x_step = if start.0 < end.0 { 1 } else { -1 };
        let y_step = if start.1 < end.1 { 1 } else { -1 };

        let mut err = dx + dy;

        loop {
            self.paint_point(Point::new(start.0 as f32, start.1 as f32), color);

            if err * 2 >= dy {
                if start.0 == end.0 {
                    break;
                }

                err += dy;
                start.0 += x_step;
            }

            if err * 2 <= dx {
                if start.1 == end.1 {
                    break;
                }

                err += dx;
                start.1 += y_step;
            }
        }
    }

    fn paint_point(&mut self, point: Point, color: u32) {
        if point.x >= self.width as f32 - 1.0 || point.y >= self.height as f32 - 1.0 {
            return;
        }

        assert!(
            (point.x as usize) < self.width,
            "{} < {}",
            point.x,
            self.width
        );
        assert!((point.y as usize) < self.height);

        let end = self.width * self.height - 1;
        let idx = point.x as usize + (end - self.width) - point.y as usize * self.height;

        self.buffer[(idx as usize).min(self.width * self.height - 1)] = color;
    }

    pub fn draw_cubic_bezier_curve(&mut self, curve: CubicBezierCurve, color: u32) {
        let mut t = 0.0_f32;

        while t < 1.0 {
            let p = curve.basis(t);

            self.paint_point(p, color);

            t += 0.0001;
        }
    }

    pub fn draw_image<'a>(
        &mut self,
        image: &ImageXObject<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<()> {
        let pixel_data = decode_stream(&image.stream.stream, &image.stream.dict, resolver)?;

        assert!(pixel_data.len() % 3 == 0);

        let rgb_data = pixel_data
            .chunks_exact(3)
            .map(|chunk| u32::from_le_bytes([chunk[2], chunk[1], chunk[0], 255]))
            .collect::<Vec<u32>>();

        for i in 0..self.height {
            let start = i * self.width;
            let end = start + (image.width as usize).min(self.width);

            if end > self.width * self.height {
                break;
            }

            let image_start = i * image.width as usize;
            let image_end = image_start + (end - start);

            if image_end > image.width as usize * image.height as usize {
                break;
            }

            self.buffer
                .get_mut(start..end)
                .unwrap()
                .copy_from_slice(&rgb_data[image_start..image_end]);
        }

        Ok(())
    }

    pub fn draw(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .unwrap();
    }
}
