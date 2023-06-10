mod color_space;
mod device_n;
mod icc;
mod indexed;

pub use color_space::{ColorSpace, ColorSpaceName};

pub struct Color;

impl Color {
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_ff_00_00;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_00_00_ff;
}
