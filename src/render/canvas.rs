use crate::{error::PdfResult, filter::decode_stream, resolve::Resolve, xobject::ImageXObject};

use minifb::{Key, Window, WindowOptions};

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
            buffer: vec![0; width * height],
            window,
        }
    }

    pub fn draw_image(
        &mut self,
        image: &ImageXObject,
        resolver: &mut dyn Resolve,
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
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)
                .unwrap();
        }
    }
}
