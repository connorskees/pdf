use std::{collections::HashMap, rc::Rc};

use crate::{
    catalog::{assert_len, MetadataStream},
    error::PdfResult,
    filter::decode_stream,
    function::Function,
    icc_profile::IccProfile,
    objects::{Name, Object},
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
pub struct DeviceNColorSpace<'a> {
    pub names: Vec<Name>,
    pub alternate_space: Rc<ColorSpace<'a>>,
    pub tint_transform: Function<'a>,
    pub attributes: Option<DeviceNColorSpaceAttributes<'a>>,
}

#[derive(Debug, Clone)]
pub struct SeparationColorSpace<'a> {
    pub name: Name,
    pub alternate_space: Rc<ColorSpace<'a>>,
    pub tint_transform: Function<'a>,
    pub tint: f32,
}

#[derive(Debug, Clone, FromObj)]
pub struct DeviceNColorSpaceAttributes<'a> {
    /// A name specifying the preferred treatment for the colour space. Values shall
    /// be DeviceN or NChannel
    ///
    /// Default value: DeviceN.
    // todo: enum?
    #[field("Subtype", default = Name("DeviceN".to_owned()))]
    subtype: Name,

    /// A dictionary describing the individual colorants that shall be used in the
    /// DeviceN colour space. For each entry in this dictionary, the key shall be
    /// a colorant name and the value shall be an array defining a Separation
    /// colour space for that colorant. The key shall match the colorant name
    /// given in that colour space.
    ///
    /// This dictionary provides information about the individual colorants that may
    /// be useful to some conforming readers. In particular, the alternate colour
    /// space and tint transformation function of a Separation colour space
    /// describe the appearance of that colorant alone, whereas those of a
    /// DeviceN colour space describe only the appearance of its colorants in
    /// combination.
    ///
    /// If Subtype is NChannel, this dictionary shall have entries for all spot
    /// colorants in this colour space. This dictionary may also include
    /// additional colorants not used by this colour space.
    // todo: maybe string => separationcolorspace
    #[field("Colorants")]
    colorants: Option<HashMap<String, ColorSpace<'a>>>,

    /// A dictionary that describes the process colour space whose components are
    /// included in this colour space.
    #[field("Process")]
    process: Option<DeviceNProcess<'a>>,

    /// A dictionary that specifies optional attributes of the inks that shall be
    /// used in blending calculations when used as an alternative to the tint
    /// transformation function.
    #[field("MixingHints")]
    mixing_hints: Option<DeviceNMixingHints<'a>>,
}

#[derive(Debug, Clone, FromObj)]
struct DeviceNMixingHints<'a> {
    #[field("Solidities")]
    solidities: Option<HashMap<String, f32>>,

    /// An array of colorant names, specifying the order in which inks shall be laid
    /// down. Each component in the names array of the DeviceN colour space shall
    /// appear in this array (although the order is unrelated to the order
    /// specified in the names array). This entry may also list colorants unused
    /// by this specific DeviceN instance.
    #[field("PrintingOrder")]
    printing_order: Option<Vec<Name>>,

    /// A dictionary specifying the dot gain of inks that shall be used in blending
    /// calculations when used as an alternative to the tint transformation
    /// function. Dot gain (or loss) represents the amount by which a printer’s
    /// halftone dots change as the ink spreads and is absorbed by paper.
    ///
    /// For each entry, the key shall be a colorant name, and the value shall be a
    /// function that maps values in the range 0 to 1 to values in the range 0 to
    /// 1. The dictionary may list colorants unused by this specific DeviceN
    /// instance and need not list all colorants. An entry with a key of Default
    /// shall specify a function to be used by all colorants for which a dot gain
    /// function is not explicitly specified.
    ///
    /// Conforming readers may ignore values in this dictionary when other sources of
    /// dot gain information are available, such as ICC profiles associated with
    /// the process colour space or tint transformation functions associated with
    /// individual colorants.
    #[field("DotGain")]
    dot_gain: Option<HashMap<String, Function<'a>>>,
}

#[derive(Debug, Clone, FromObj)]
struct DeviceNProcess<'a> {
    /// A name or array identifying the process colour space, which may be any device
    /// or CIE-based colour space. If an ICCBased colour space is specified, it
    /// shall provide calibration information appropriate for the process colour
    /// components specified in the names array of the DeviceN colour space.
    #[field("ColorSpace")]
    color_space: Box<ColorSpace<'a>>,

    /// An array of component names that correspond, in order, to the components of
    /// the process colour space specified in ColorSpace. For example, an RGB
    /// colour space shall have three names corresponding to red, green, and
    /// blue. The names may be arbitrary (that is, not the same as the standard
    /// names for the colour space components) and shall match those specified in
    /// the names array of the DeviceN colour space, even if all components are
    /// not present in the names array.
    #[field("Components")]
    components: Vec<Name>,
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
