use std::{fs::File, io::BufWriter, mem, path::Path as FilePath};

use bitvec::{prelude::Lsb0, slice::BitSlice};

use crate::{
    color::{Color, ColorSpace, ColorSpaceName},
    error::PdfResult,
    filter::{decode_stream, flate::BitsPerComponent},
    geometry::{CubicBezierCurve, Line, Outline, Path, Point, QuadraticBezierCurve, Subpath},
    resolve::Resolve,
    xobject::ImageXObject,
};

#[cfg(feature = "window")]
use minifb::{Key, Window, WindowOptions};

use super::FillRule;

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
    #[cfg(feature = "window")]
    window: Window,
}

fn parse_rgba(color: u32) -> [u8; 4] {
    let r = color & 0xff;
    let g = (color >> 8) & 0xff;
    let b = (color >> 16) & 0xff;

    [r as u8, g as u8, b as u8, 0xff]
}

fn apply_opacity(color: u32, opacity: f32) -> u32 {
    assert!(opacity >= 0.0 && opacity <= 1.0);

    let [red, green, blue, _] = parse_rgba(color);

    ColorSpace::DeviceRGB {
        red: ((red as f32 * opacity) + 255.0 * (1.0 - opacity)) / 255.0,
        green: ((green as f32 * opacity) + 255.0 * (1.0 - opacity)) / 255.0,
        blue: ((blue as f32 * opacity) + 255.0 * (1.0 - opacity)) / 255.0,
    }
    .as_u32()
}

impl Canvas {
    #[cfg(feature = "window")]
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

    #[cfg(not(feature = "window"))]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![u32::MAX; width * height],
        }
    }

    pub fn fill_path_non_zero_winding_number(&mut self, path: &Path, color: u32) {
        self.fill_path_even_odd(path, color)
    }

    pub fn fill_path_even_odd(&mut self, path: &Path, color: u32) {
        let bbox = path.bounding_box();

        let x = bbox.min.x as u32;
        let y = bbox.min.y as u32;

        let mut h = bbox.min.y;
        while h < bbox.height().ceil() + y as f32 {
            let mut w = bbox.min.x;
            while w < bbox.width().ceil() + x as f32 {
                let point = Point::new(w as f32 + 0.5, h as f32 + 0.5);

                let subpixel_step = 0.125;

                let left = Point::new(point.x - subpixel_step, point.y);
                let right = Point::new(point.x + subpixel_step, point.y);
                let down = Point::new(point.x, point.y - subpixel_step);
                let up = Point::new(point.x, point.y + subpixel_step);

                let upper_left = Point::new(point.x - subpixel_step, point.y + subpixel_step);
                let upper_right = Point::new(point.x + subpixel_step, point.y + subpixel_step);
                let lower_left = Point::new(point.x - subpixel_step, point.y - subpixel_step);
                let lower_right = Point::new(point.x + subpixel_step, point.y - subpixel_step);

                let end = Point::new(point.x + bbox.max.x + 1.0, point.y + bbox.max.y + 1.0);

                if path.intersects_line_even_odd(Line::new(point, end)) {
                    self.paint_point(point, color);
                } else {
                    let count = [
                        left,
                        right,
                        down,
                        up,
                        upper_left,
                        upper_right,
                        lower_left,
                        lower_right,
                    ]
                    .into_iter()
                    .filter(|point| path.intersects_line_even_odd(Line::new(*point, end)))
                    .count();

                    self.paint_point(point, apply_opacity(color, count as f32 / 8.0));
                }

                w += 1.0;
            }

            h += 1.0;
        }
    }

    pub fn fill_path(&mut self, path: &Path, color: u32, fill_rule: FillRule) {
        match fill_rule {
            FillRule::EvenOdd => self.fill_path_even_odd(path, color),
            FillRule::NonZeroWindingNumber => self.fill_path_non_zero_winding_number(path, color),
        }
    }

    pub fn fill_outline_even_odd(&mut self, outline: &Outline, color: u32) {
        // todo: optimize to not require allocation or iteration
        let subpaths = outline
            .paths
            .iter()
            .flat_map(|path| path.subpaths.clone())
            .collect();

        let path = Path::from_subpaths(subpaths);

        self.fill_path_even_odd(&path, color);
    }

    pub fn stroke_outline(&mut self, outline: &Outline, color: u32) {
        for path in &outline.paths {
            self.stroke_path(path, color);
        }
    }

    /// Largely to assist in debugging
    pub fn stroke_bounding_box(&mut self, outline: &Outline) {
        let bbox = outline.bounding_box();

        let mut path = Path::new(bbox.min);
        path.line_to(Point::new(bbox.max.x, bbox.min.y));
        path.line_to(bbox.max);
        path.line_to(Point::new(bbox.min.x, bbox.max.y));
        path.close_path();

        self.stroke_path(&path, Color::RED);
    }

    pub fn stroke_path(&mut self, path: &Path, color: u32) {
        for &subpath in &path.subpaths {
            match subpath {
                Subpath::Line(line) => self.stroke_line(line, color),
                Subpath::Quadratic(curve) => self.stroke_quadratic_bezier_curve(curve, color),
                Subpath::Cubic(curve) => self.stroke_cubic_bezier_curve(curve, color),
            }
        }
    }

    pub fn stroke_line_wu(&mut self, mut line: Line, color: u32) {
        fn fractional_part(f: f32) -> f32 {
            f - f.floor()
        }

        fn r_fractional_part(f: f32) -> f32 {
            1.0 - fractional_part(f)
        }

        let steep = (line.end.y - line.start.y).abs() > (line.end.x - line.start.x).abs();

        if steep {
            mem::swap(&mut line.start.x, &mut line.start.y);
            mem::swap(&mut line.end.x, &mut line.end.y);
        }

        if line.start.x > line.end.x {
            mem::swap(&mut line.start.x, &mut line.end.x);
            mem::swap(&mut line.start.y, &mut line.end.y);
        }

        let dx = line.end.x - line.start.x;
        let dy = line.end.y - line.start.y;

        let gradient = if fuzzy_eq(dx, 0.0) { 1.0 } else { dy / dx };

        let xend = line.start.x.round();
        let yend = line.start.y + gradient * (xend - line.start.x);

        let xgap = r_fractional_part(line.start.x + 0.5);

        let xpxl1 = xend;
        let ypxl1 = yend.floor();

        if steep {
            self.paint_point(
                Point::new(ypxl1, xpxl1),
                apply_opacity(color, r_fractional_part(yend) * xgap),
            );

            self.paint_point(
                Point::new(ypxl1 + 1.0, xpxl1),
                apply_opacity(color, fractional_part(yend) * xgap),
            );
        } else {
            self.paint_point(
                Point::new(xpxl1, ypxl1),
                apply_opacity(color, r_fractional_part(yend) * xgap),
            );

            self.paint_point(
                Point::new(xpxl1, ypxl1 + 1.0),
                apply_opacity(color, fractional_part(yend) * xgap),
            );
        }

        let mut intery = yend + gradient;

        let xend = line.end.x.round();
        let yend = line.end.y + gradient * (xend - line.end.x);

        let xgap = fractional_part(line.end.x + 0.5);

        let xpxl2 = xend;
        let ypxl2 = yend.floor();

        if steep {
            self.paint_point(
                Point::new(ypxl2, xpxl2),
                apply_opacity(color, r_fractional_part(yend) * xgap),
            );
            self.paint_point(
                Point::new(ypxl2 + 1.0, xpxl2),
                apply_opacity(color, fractional_part(yend) * xgap),
            );
        } else {
            self.paint_point(
                Point::new(xpxl2, ypxl2),
                apply_opacity(color, r_fractional_part(yend) * xgap),
            );

            self.paint_point(
                Point::new(xpxl2, ypxl2 + 1.0),
                apply_opacity(color, fractional_part(yend) * xgap),
            );
        }

        if steep {
            for x in (xpxl1 as i32 + 1)..(xpxl2 as i32 - 1) {
                self.paint_point(
                    Point::new(intery.floor(), x as f32),
                    apply_opacity(color, r_fractional_part(intery)),
                );
                self.paint_point(
                    Point::new(intery.floor() + 1.0, x as f32),
                    apply_opacity(color, fractional_part(intery)),
                );

                intery += gradient
            }
        } else {
            for x in (xpxl1 as i32 + 1)..(xpxl2 as i32 - 1) {
                self.paint_point(
                    Point::new(x as f32, intery.floor()),
                    apply_opacity(color, r_fractional_part(intery)),
                );
                self.paint_point(
                    Point::new(x as f32, intery.floor() + 1.0),
                    apply_opacity(color, fractional_part(intery)),
                );

                intery += gradient
            }
        }
    }

    pub fn stroke_line(&mut self, line: Line, color: u32) {
        // todo: maybe do this optimization
        let mut start = (
            (line.start.x.round() as i32), //.min(self.width as i32),
            (line.start.y.round() as i32), //.min(self.height as i32),
        );
        let end = (
            (line.end.x.round() as i32), //.min(self.width as i32),
            (line.end.y.round() as i32), //.min(self.height as i32),
        );

        let dx = (end.0 - start.0).abs();
        let dy = -(end.1 - start.1).abs();

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
        if point.x >= self.width as f32 || point.y >= self.height as f32 {
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

        // todo: what does this condition imply
        if point.y as usize * self.width > point.x as usize + (end - self.width) {
            return;
        }

        let idx = point.x as usize + (end - self.width) - point.y as usize * self.width;

        self.buffer[idx.min(self.width * self.height - 1)] = color;
    }

    pub fn stroke_quadratic_bezier_curve(&mut self, curve: QuadraticBezierCurve, color: u32) {
        let mut t = 0.0_f32;

        while t < 1.0 {
            let p = curve.basis(t);

            self.paint_point(p, color);

            t += 0.001;
        }
    }

    pub fn stroke_cubic_bezier_curve(&mut self, curve: CubicBezierCurve, color: u32) {
        let mut t = 0.0_f32;

        while t < 1.0 {
            let p = curve.basis(t);

            self.paint_point(p, color);

            t += 0.001;
        }
    }

    pub fn draw_image<'a>(
        &mut self,
        image: &ImageXObject<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<()> {
        let pixel_data = decode_stream(&image.stream.stream, &image.stream.dict, resolver)?;

        let rgb_data = match image.color_space.as_ref().map(ColorSpace::name) {
            Some(ColorSpaceName::DeviceGray) => match image.bits_per_component {
                Some(BitsPerComponent::Eight) => pixel_data
                    .iter()
                    .map(|b| ColorSpace::DeviceGray(*b as f32).as_u32())
                    .collect::<Vec<u32>>(),
                Some(BitsPerComponent::One) => BitSlice::<u8, Lsb0>::from_slice(&pixel_data)
                    .iter()
                    .by_vals()
                    .map(|b| ColorSpace::DeviceGray(b as u8 as f32).as_u32())
                    .collect::<Vec<u32>>(),
                _ => todo!(),
            },
            Some(ColorSpaceName::DeviceCMYK) => pixel_data
                .chunks_exact(4)
                .map(|chunk| {
                    ColorSpace::DeviceCMYK {
                        cyan: chunk[3] as f32,
                        magenta: chunk[2] as f32,
                        yellow: chunk[1] as f32,
                        key: chunk[0] as f32,
                    }
                    .as_u32()
                })
                .collect::<Vec<u32>>(),
            Some(ColorSpaceName::CalGray) => todo!(),
            Some(ColorSpaceName::CalRGB) => todo!(),
            Some(ColorSpaceName::Lab) => todo!(),
            Some(ColorSpaceName::ICCBased) => todo!(),
            Some(ColorSpaceName::Indexed) => todo!(),
            Some(ColorSpaceName::Pattern) => todo!(),
            Some(ColorSpaceName::Separation) => todo!(),
            Some(ColorSpaceName::DeviceN) => todo!(),
            Some(ColorSpaceName::DeviceRGB) | None => pixel_data
                .chunks_exact(3)
                .map(|chunk| {
                    ColorSpace::DeviceRGB {
                        red: chunk[2] as f32,
                        green: chunk[1] as f32,
                        blue: chunk[0] as f32,
                    }
                    .as_u32()
                })
                .collect::<Vec<u32>>(),
        };

        assert_eq!(rgb_data.len() % image.width as usize, 0);

        for i in 0..self.height {
            let start: usize = i * self.width;
            let end = start + (image.width as usize).min(self.width);

            if end > self.width * self.height {
                break;
            }

            let image_start = i * image.width as usize;
            let image_end = image_start + (end - start);

            if image_end > image.width as usize * image.height as usize
                || image_end >= rgb_data.len()
            {
                break;
            }

            self.buffer
                .get_mut(start..end)
                .unwrap()
                .copy_from_slice(&rgb_data[image_start..image_end]);
        }

        Ok(())
    }

    fn render_to_image(&mut self, p: impl AsRef<FilePath>) {
        let file = File::create(p).unwrap();
        let w = &mut BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();

        let data = self
            .buffer
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        writer.write_image_data(&data).unwrap();
    }

    pub fn draw(&mut self) {
        #[cfg(feature = "window")]
        {
            while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
                self.refresh();
            }
        }

        #[cfg(not(feature = "window"))]
        {
            self.render_to_image("/root/pdf/foo.png");
        }
    }

    pub fn refresh(&mut self) {
        #[cfg(feature = "window")]
        {
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)
                .unwrap();
        }
    }
}
