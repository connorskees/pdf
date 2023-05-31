use crate::{
    catalog::MetadataStream,
    color::ColorSpace,
    error::PdfResult,
    filter::flate::BitsPerComponent,
    objects::Object,
    optional_content::{OptionalContent, OptionalContentGroup},
    resources::graphics_state_parameters::RenderingIntent,
    stream::Stream,
    FromObj, Resolve,
};

use super::OpenPrepressInterface;

#[derive(Debug, Clone, FromObj)]
pub struct ImageXObject<'a> {
    /// The width of the image, in samples
    #[field("Width")]
    pub width: u32,

    /// The height of the image, in samples
    #[field("Height")]
    pub height: u32,

    #[field]
    pub stream: Stream<'a>,

    /// The colour space in which image samples shall be specified; it can be
    /// any type of colour space except Pattern.
    ///
    /// Required for images, except those that use the JPXDecode filter
    ///
    /// If the image uses the JPXDecode filter, this entry may be present:
    ///   * If ColorSpace is present, any colour space specifications in the
    ///      JPEG2000 data shall be ignored.
    ///   * If ColorSpace is absent, the colour space specifications in the
    ///      JPEG2000 data shall be used. The Decode array shall also be
    ///      ignored unless ImageMask is true
    ///
    #[field("ColorSpace")]
    pub color_space: Option<ColorSpace<'a>>,

    /// The number of bits used to represent each colour component. Only a
    /// single value shall be specified; the number of bits shall be the same
    /// for all colour components. The value shall be 1, 2, 4, 8, or 16. If
    /// ImageMask is true, this entry is optional, but if specified, its value
    /// shall be 1.
    ///
    /// If the image stream uses a filter, the value of BitsPerComponent shall
    /// be consistent with the size of the data samples that the filter delivers.
    /// In particular, a CCITTFaxDecode or JBIG2Decode filter shall always deliver
    /// 1-bit samples, a RunLengthDecode or DCTDecode filter shall always deliver
    /// 8-bit samples, and an LZWDecode or FlateDecode filter shall deliver
    /// samples of a specified size if a predictor function is used.
    ///
    /// Required for images, except those that use the JPXDecode filter
    ///
    /// If the image stream uses the JPXDecode filter, this entry is optional and
    /// shall be ignored if present. The bit depth is determined by the conforming
    /// reader in the process of decoding the JPEG2000 image
    #[field("BitsPerComponent")]
    pub bits_per_component: Option<BitsPerComponent>,

    /// The name of a colour rendering intent to be used in rendering the image
    ///
    /// Default value: the current rendering intent in the graphics state
    #[field("Intent")]
    pub intent: Option<RenderingIntent>,

    /// A flag indicating whether the image shall be treated as an image mask
    ///
    /// If this flag is true, the value of BitsPerComponent shall be 1 and Mask
    /// and ColorSpace shall not be specified; unmasked areas shall be painted
    /// using the current nonstroking colour
    ///
    /// Default value: false
    #[field("ImageMask", default = false)]
    pub image_mask: bool,

    /// An image XObject defining an image mask to be applied to this image, or an
    /// array specifying a range of colours to be applied to it as a colour key
    /// mask. If ImageMask is true, this entry shall not be present
    #[field("Mask")]
    pub mask: Option<ImageMask>,

    /// An array of numbers describing how to map image samples into the range of
    /// values appropriate for the image's colour space. If ImageMask is true, the
    /// array shall be either [0 1] or [1 0]; otherwise, its length shall be twice
    /// the number of colour components required by ColorSpace. If the image uses
    /// the JPXDecode filter and ImageMask is false, Decode shall be ignored by a
    /// conforming reader
    #[field("Decode")]
    pub decode: Option<Vec<f32>>,

    /// A flag indicating whether image interpolation shall be performed by a conforming
    /// reader
    ///
    /// Default value: false
    #[field("Interpolate", default = false)]
    pub interpolate: bool,

    /// An array of alternate image dictionaries for this image. The order of elements
    /// within the array shall have no significance. This entry shall not be present
    /// in an image XObject that is itself an alternate image
    #[field("Alternates")]
    pub alternates: Option<Vec<AlternateImage<'a>>>,

    /// A subsidiary image XObject defining a softmask image that shall be used as a source
    /// of mask shape or mask opacity values in the transparent imaging model. The alpha
    /// source parameter in the graphics state determines whether the mask values shall be
    /// interpreted as shape or opacity.
    ///
    /// If present, this entry shall override the current soft mask in the graphics state,
    /// as well as the image's Mask entry, if any. However, the other transparency-related
    /// graphics state parameters -- blend mode and alpha constant -- shall remain in effect.
    /// If SMask is absent, the image shall have no associated soft mask (although the current
    /// soft mask in the graphics state may still apply)
    #[field("SMask")]
    pub s_mask: Option<SoftMask<'a>>,

    /// A code specifying how soft-mask information encoded with image samples shall be used:
    ///   0 If present, encoded soft-mask image information shall be ignored.
    ///   1 The image's data stream includes encoded soft-mask values. A conforming reader
    ///       may create a soft-mask image from the information to be used as a source of mask
    ///       shape or mask opacity in the transparency imaging model.
    ///   2 The image's data stream includes colour channels that have been preblended with a
    ///       background; the image data also includes an opacity channel. A conforming reader
    ///       may create a soft-mask image with a Matte entry from the opacity channel information
    ///       to be used as a source of mask shape or mask opacity in the transparency model.
    ///
    /// If this entry has a nonzero value, SMask shall not be specified.
    ///
    /// Default value: 0.
    #[field("SMaskInData", default = 0)]
    pub s_mask_in_data: i32,

    /// The name by which this image XObject is referenced in the XObject subdictionary of the
    /// current resource dictionary.
    ///
    /// This entry is obsolescent and shall no longer be used
    #[field("Name")]
    pub name: Option<String>,

    /// The integer key of the image's entry in the structural parent tree
    #[field("StructParent")]
    pub struct_parent: Option<i32>,

    /// The digital identifier of the image's parent Web Capture content set
    #[field("ID")]
    pub id: Option<String>,

    /// An OPI version dictionary for the image. If ImageMask is true, this entry shall be ignored
    #[field("OPI")]
    pub opi: Option<OpenPrepressInterface>,

    /// A metadata stream containing metadata for the image
    #[field("Metadata")]
    pub metadata: Option<MetadataStream<'a>>,

    /// An optional content group or optional content membership dictionary, specifying the optional
    /// content properties for this image XObject. Before the image is processed by a conforming reader,
    /// its visibility shall be determined based on this entry. If it is determined to be invisible,
    /// the entire image shall be skipped, as if there were no Do operator to invoke it
    #[field("OC")]
    pub oc: Option<OptionalContent>,
}

#[derive(Debug, Clone, FromObj)]
pub struct SoftMask<'a> {
    #[field("Width")]
    width: u32,

    #[field("Height")]
    height: u32,

    #[field("ColorSpace")]
    color_space: ColorSpace<'a>,

    #[field("BitsPerComponent")]
    bits_per_component: BitsPerComponent,

    /// Default value: [0 1]
    #[field("Decode", default = vec![0.0, 1.0])]
    decode: Vec<f32>,

    #[field("Interpolate")]
    interpolate: Option<bool>,

    /// An array of component values specifying the matte colour with which the image data in
    /// the parent image shall have been preblended. The array shall consist of n numbers, where
    /// n is the number of components in the colour space specified by the ColorSpace entry in
    /// the parent image's image dictionary; the numbers shall be valid colour components in that
    /// colour space. If this entry is absent, the image data shall not be preblended
    // todo: type
    #[field("Matte")]
    matte: Option<Vec<Object<'a>>>,

    #[field]
    stream: Stream<'a>,
}

#[derive(Debug, Clone)]
pub struct AlternateImage<'a> {
    /// The image XObject for the alternate image
    image: ImageXObject<'a>,

    /// A flag indicating whether this alternate image shall be the default
    /// version to be used for printing
    ///
    /// At most one alternate for a given base image shall be so designated.
    /// If no alternate has this entry set to true, the base image shall be used
    /// for printing by a conforming reader
    default_for_printing: Option<bool>,

    /// An optional content group or optional content membership dictionary
    /// that facilitates the selection of which alternate image to use
    // todo: optional content membership
    oc: Option<OptionalContentGroup>,
}

impl<'a> FromObj<'a> for AlternateImage<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut stream = resolver.assert_stream(obj)?;
        let dict = &mut stream.dict.other;

        let default_for_printing = dict.get_bool("DefaultForPrinting", resolver)?;
        let oc = dict.get("OC", resolver)?;

        let image = ImageXObject::from_obj(Object::Stream(stream), resolver)?;

        Ok(Self {
            image,
            default_for_printing,
            oc,
        })
    }
}

#[derive(Debug, Clone, FromObj)]
pub struct ImageMask;
