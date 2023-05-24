use crate::{catalog::assert_len, error::PdfResult, objects::Object, resolve::Resolve};

pub struct Color;

impl Color {
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_ff_00_00;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_00_00_ff;
}

#[derive(Debug, Clone)]
pub enum ColorSpace {
    // Device
    DeviceGray(f32),
    DeviceRGB {
        red: f32,
        green: f32,
        blue: f32,
    },
    DeviceCMYK {
        cyan: f32,
        magenta: f32,
        yellow: f32,
        key: f32,
    },

    // CIE-based
    CalGray {
        a: f32,
    },
    CalRGB {
        a: f32,
        b: f32,
        c: f32,
    },
    Lab {
        a: f32,
        b: f32,
        c: f32,
    },
    ICCBased(Vec<f32>),

    // Special
    Indexed(u32),
    Pattern,
    Separation,
    DeviceN(Vec<f32>),
}

impl ColorSpace {
    pub fn init(name: ColorSpaceName) -> Self {
        match name {
            ColorSpaceName::DeviceGray => ColorSpace::DeviceGray(0.0),
            ColorSpaceName::DeviceRGB => ColorSpace::DeviceRGB {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
            ColorSpaceName::DeviceCMYK => ColorSpace::DeviceCMYK {
                cyan: 0.0,
                magenta: 0.0,
                yellow: 0.0,
                key: 1.0,
            },
            ColorSpaceName::CalGray => ColorSpace::CalGray { a: 0.0 },
            ColorSpaceName::CalRGB => ColorSpace::CalRGB {
                a: 0.0,
                b: 0.0,
                c: 0.0,
            },
            ColorSpaceName::Lab => todo!(),
            ColorSpaceName::ICCBased => todo!(),
            ColorSpaceName::Indexed => ColorSpace::Indexed(0),
            ColorSpaceName::Pattern => todo!(),
            ColorSpaceName::Separation => todo!(),
            ColorSpaceName::DeviceN => todo!(),
        }
    }

    pub fn name(&self) -> ColorSpaceName {
        match self {
            ColorSpace::DeviceGray(..) => ColorSpaceName::DeviceGray,
            ColorSpace::DeviceRGB { .. } => ColorSpaceName::DeviceRGB,
            ColorSpace::DeviceCMYK { .. } => ColorSpaceName::DeviceCMYK,
            ColorSpace::CalGray { .. } => ColorSpaceName::CalGray,
            ColorSpace::CalRGB { .. } => ColorSpaceName::CalRGB,
            ColorSpace::Lab { .. } => ColorSpaceName::Lab,
            ColorSpace::ICCBased(..) => ColorSpaceName::ICCBased,
            ColorSpace::Indexed(..) => ColorSpaceName::Indexed,
            ColorSpace::Pattern { .. } => ColorSpaceName::Pattern,
            ColorSpace::Separation { .. } => ColorSpaceName::Separation,
            ColorSpace::DeviceN(..) => ColorSpaceName::DeviceN,
        }
    }

    /// For the framebuffer we currently use, this appears to be in ARGB format
    ///
    /// This may change in the future
    pub fn as_u32(&self) -> u32 {
        match self {
            // todo: argb
            Self::DeviceGray(n) => *n as u32,
            &Self::DeviceRGB { red, green, blue } => {
                let r = (red * 255.0) as u32;
                let g = (green * 255.0) as u32;
                let b = (blue * 255.0) as u32;

                (0xff << 24) | (r << 16) | (g << 8) | b
            }
            &Self::DeviceCMYK {
                cyan,
                magenta,
                yellow,
                key,
            } => {
                let r = (255.0 * (1.0 - cyan) * (1.0 - key)) as u32;
                let g = (255.0 * (1.0 - magenta) * (1.0 - key)) as u32;
                let b = (255.0 * (1.0 - yellow) * (1.0 - key)) as u32;

                (0xff << 24) | (r << 16) | (g << 8) | b
            }
            c => todo!("unimplemented color space: {:?}", c),
        }
    }

    pub(crate) fn from_obj<'a>(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        match resolver.resolve(obj)? {
            Object::Name(name) => Ok(ColorSpace::init(ColorSpaceName::from_str(&name)?)),
            Object::Array(arr) => {
                let name = resolver.assert_name(arr[0].clone())?;

                match ColorSpaceName::from_str(&name)? {
                    ColorSpaceName::DeviceGray => todo!(),
                    ColorSpaceName::DeviceRGB => todo!(),
                    ColorSpaceName::DeviceCMYK => todo!(),
                    ColorSpaceName::CalGray => todo!(),
                    ColorSpaceName::CalRGB => todo!(),
                    ColorSpaceName::Lab => todo!(),
                    ColorSpaceName::ICCBased => todo!(),
                    ColorSpaceName::Indexed => {
                        assert_len(&arr, 4)?;

                        // let lookup = arr.pop().unwrap();
                        // let hival = arr.pop().unwrap();
                        // let base = arr.pop().unwrap();

                        todo!()
                    }
                    ColorSpaceName::Pattern => todo!(),
                    ColorSpaceName::Separation => todo!(),
                    ColorSpaceName::DeviceN => todo!(),
                }
            }
            _ => todo!(),
        }
    }
}

#[pdf_enum]
pub enum ColorSpaceName {
    DeviceGray = "DeviceGray",
    DeviceRGB = "DeviceRGB",
    DeviceCMYK = "DeviceCMYK",
    CalGray = "CalGray",
    CalRGB = "CalRGB",
    Lab = "Lab",
    ICCBased = "ICCBased",
    Indexed = "Indexed",
    Pattern = "Pattern",
    Separation = "Separation",
    DeviceN = "DeviceN",
}
