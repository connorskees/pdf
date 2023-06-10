use std::{collections::HashMap, rc::Rc};

use crate::{function::Function, objects::Name};

use super::ColorSpace;

#[derive(Debug, Clone)]
pub struct DeviceNColorSpace<'a> {
    pub names: Vec<Name>,
    pub alternate_space: Rc<ColorSpace<'a>>,
    pub tint_transform: Function<'a>,
    pub attributes: Option<DeviceNColorSpaceAttributes<'a>>,
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
