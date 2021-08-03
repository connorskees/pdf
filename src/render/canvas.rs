use crate::xobject::ImageXObject;

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

    pub fn draw_image(&mut self, _image: &ImageXObject) {}

    pub fn draw(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)
                .unwrap();
        }
    }
}
