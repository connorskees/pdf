mod color_space;
mod device_n;
mod icc;
mod indexed;

pub use color_space::{ColorSpace, ColorSpaceName};

pub struct Color;

impl Color {
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const WHITE: u32 = 0xff_ff_ff_ff;
    pub const GRAY_75: u32 = 0xff_40_40_40;
    pub const GRAY_50: u32 = 0xff_80_80_80;
    pub const GRAY_25: u32 = 0xff_d7_d7_d7;
    pub const RED: u32 = 0xff_ff_00_00;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_00_00_ff;
}
