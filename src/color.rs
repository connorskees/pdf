use std::rc::Rc;

use crate::{
    catalog::{assert_len, MetadataStream},
    error::PdfResult,
    filter::decode_stream,
    icc_profile::IccProfile,
    objects::Object,
    resources::pattern::Pattern,
    stream::Stream,
    FromObj, Resolve,
};

pub struct Color;

impl Color {
    pub const BLACK: u32 = 0xff_00_00_00;
    pub const RED: u32 = 0xff_ff_00_00;
    pub const GREEN: u32 = 0xff_00_ff_00;
    pub const BLUE: u32 = 0xff_00_00_ff;
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
    Indexed(u32),
    Pattern(Option<Rc<Pattern<'a>>>),
    Separation,
    DeviceN(Vec<f32>),
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
            ColorSpaceName::Indexed => ColorSpace::Indexed(0),
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
            ColorSpace::Indexed(..) => ColorSpaceName::Indexed,
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

                (0xff << 24) | (r << 16) | (g << 8) | b
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
                        assert_eq!(
                            icc_profile.header.colour_space.0, *b"RGB ",
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

#[derive(Debug, Clone, FromObj)]
pub struct IccStream<'a> {
    /// The number of colour components in the colour space described by the ICC
    /// profile data. This number shall match the number of components
    /// actually in the ICC profile.
    #[field("N")]
    pub num_of_color_components: i32,

    /// An alternate colour space that shall be used in case the one specified in
    /// the stream data is not supported.
    ///
    /// Non-conforming readers may use this colour space. The alternate space may
    /// be any valid colour space (except a Pattern colour space) that has
    /// the number of components specified by N. If this entry is omitted
    /// and the conforming reader does not understand the ICC profile data,
    /// the colour space that shall be used is DeviceGray, DeviceRGB, or
    /// DeviceCMYK, depending on whether the value of N is 1, 3, or 4,
    /// respectively.
    ///
    /// There shall not be conversion of source colour values, such as a tint
    /// transformation, when using the alternate colour space. Colour
    /// values within the range of the ICCBased colour space might not be
    /// within the range of the alternate colour space. In this case, the
    /// nearest values within the range of the alternate space shall be
    /// substituted.
    #[field("Alternate")]
    pub alternate: Option<Box<ColorSpace<'a>>>,

    /// An array of 2 × N numbers [min0 max0 min1 max1 ...] that shall specify the
    /// minimum and maximum valid values of the corresponding colour components.
    /// These values shall match the information in the ICC profile.
    ///
    /// Default value: [0.0 1.0 0.0 1.0 ...]
    // todo: prettier way to do this?
    // todo: special struct for this
    #[field("Range", default = [0.0_f32, 1.0].into_iter().cycle().take(num_of_color_components as usize * 2).collect())]
    pub range: Vec<f32>,

    /// A metadata stream that shall contain metadata for the colour space
    #[field("Metadata")]
    pub metadata: Option<MetadataStream<'a>>,

    #[field]
    pub stream: Stream<'a>,
}

#[pdf_enum(Integer)]
enum IccColorComponentNum {
    One = 1,
    Three = 3,
    Four = 4,
}
