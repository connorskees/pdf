use std::rc::Rc;

use crate::{
    catalog::assert_len,
    error::PdfResult,
    filter::decode_stream,
    function::Function,
    icc_profile::IccProfile,
    objects::{Name, Object},
    resources::pattern::Pattern,
    FromObj, Resolve,
};

use super::{
    device_n::{DeviceNColorSpace, DeviceNColorSpaceAttributes},
    icc::IccStream,
    indexed::{IndexedColorSpace, IndexedLookupTable},
};

#[derive(Debug, Clone)]
pub struct SeparationColorSpace<'a> {
    pub name: Name,
    pub alternate_space: Rc<ColorSpace<'a>>,
    pub tint_transform: Function<'a>,
    pub tint: f32,
}

#[derive(Debug, Clone)]
pub enum ColorSpace<'a> {
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
    IccBased {
        stream: Rc<IccStream<'a>>,
        channels: Vec<f32>,
    },

    // Special
    Indexed {
        index: u32,
        space: Rc<IndexedColorSpace<'a>>,
    },
    Pattern(Option<Rc<Pattern<'a>>>),
    Separation(SeparationColorSpace<'a>),
    DeviceN(DeviceNColorSpace<'a>),
}

impl<'a> ColorSpace<'a> {
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
            ColorSpaceName::Indexed => todo!(),
            ColorSpaceName::Pattern => ColorSpace::Pattern(None),
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
            ColorSpace::IccBased { .. } => ColorSpaceName::ICCBased,
            ColorSpace::Indexed { .. } => ColorSpaceName::Indexed,
            ColorSpace::Pattern { .. } => ColorSpaceName::Pattern,
            ColorSpace::Separation { .. } => ColorSpaceName::Separation,
            ColorSpace::DeviceN(..) => ColorSpaceName::DeviceN,
        }
    }

    #[allow(unused)]
    fn blend(&self, background: Self) -> Self {
        todo!()
    }

    /// For the framebuffer we currently use, this is in 0RGB format
    ///
    /// This may change in the future
    pub fn as_u32(&self) -> u32 {
        match self {
            &Self::DeviceGray(n) => {
                let n = n.round() as u32;

                (0xff << 24) | (n << 16) | (n << 8) | n
            }
            &Self::DeviceRGB { red, green, blue } => {
                let r = (red * 255.0).round() as u32;
                let g = (green * 255.0).round() as u32;
                let b = (blue * 255.0).round() as u32;

                (0xff << 24) | (b << 16) | (g << 8) | r
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

                (0xff << 24) | (b << 16) | (g << 8) | r
            }
            Self::Pattern(..) => {
                // todo: we just set color to red for now
                let r = (1.0 * 255.0) as u32;
                let g = (0.0 * 255.0) as u32;
                let b = (0.0 * 255.0) as u32;

                (0xff << 24) | (b << 16) | (g << 8) | r
            }
            Self::IccBased { stream, channels } => {
                // ensure we don't silently render colors we don't support
                assert_eq!(stream.num_of_color_components, 3);
                assert_eq!(channels.len(), 3);

                let r = (channels[0] * 255.0) as u32;
                let g = (channels[1] * 255.0) as u32;
                let b = (channels[2] * 255.0) as u32;

                (0xff << 24) | (b << 16) | (g << 8) | r
            }
            Self::Separation(space) => {
                todo!("{:#?}", space)
            }
            c => todo!("unimplemented color space: {:#?}", c),
        }
    }
}

impl<'a> FromObj<'a> for ColorSpace<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
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
                    ColorSpaceName::ICCBased => {
                        assert_len(&arr, 2)?;

                        let icc_stream = Rc::new(IccStream::from_obj(arr[1].clone(), resolver)?);

                        let stream = decode_stream(
                            &icc_stream.stream.stream,
                            &icc_stream.stream.dict,
                            resolver,
                        )?;

                        let icc_profile = IccProfile::new(&stream)?;
                        assert!(
                            matches!(&icc_profile.header.colour_space.0, b"RGB " | b"GRAY"),
                            "unimplemented ICC color profile: {:?}",
                            icc_profile.header.colour_space
                        );

                        Ok(ColorSpace::IccBased {
                            // todo: should actually be the lower bound of the
                            // Range for each channel instead of 0.0
                            channels: vec![0.0; icc_stream.num_of_color_components as usize],
                            stream: icc_stream,
                        })
                    }
                    ColorSpaceName::Indexed => {
                        assert_len(&arr, 4)?;

                        let base = ColorSpace::from_obj(arr[1].clone(), resolver)?;
                        let hival = u8::try_from(u32::from_obj(arr[2].clone(), resolver)?)?;
                        let lookup = IndexedLookupTable::from_obj(arr[3].clone(), resolver)?;

                        let space = Rc::new(IndexedColorSpace {
                            base,
                            hival,
                            lookup,
                        });

                        Ok(ColorSpace::Indexed { index: 0, space })
                    }
                    ColorSpaceName::Pattern => todo!(),
                    ColorSpaceName::Separation => {
                        assert_len(&arr, 4)?;

                        let name = <Name>::from_obj(arr[1].clone(), resolver)?;
                        let alternate_space = ColorSpace::from_obj(arr[2].clone(), resolver)?;
                        let tint_transform = Function::from_obj(arr[3].clone(), resolver)?;

                        let space = SeparationColorSpace {
                            name,
                            alternate_space: Rc::new(alternate_space),
                            tint_transform,
                            tint: 1.0,
                        };

                        Ok(ColorSpace::Separation(space))
                    }
                    ColorSpaceName::DeviceN => {
                        let names = <Vec<Name>>::from_obj(arr[1].clone(), resolver)?;
                        let alternate_space = ColorSpace::from_obj(arr[2].clone(), resolver)?;
                        let tint_transform = Function::from_obj(arr[3].clone(), resolver)?;
                        let attributes = if let Some(obj) = arr.get(4) {
                            Some(DeviceNColorSpaceAttributes::from_obj(
                                obj.clone(),
                                resolver,
                            )?)
                        } else {
                            None
                        };

                        let space = DeviceNColorSpace {
                            names,
                            alternate_space: Rc::new(alternate_space),
                            tint_transform,
                            attributes,
                        };

                        Ok(ColorSpace::DeviceN(space))
                    }
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
