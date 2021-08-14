use crate::{
    catalog::MetadataStream,
    error::PdfResult,
    filter::flate::BitsPerComponent,
    objects::{Dictionary, Object},
    optional_content::OptionalContent,
    resources::graphics_state_parameters::RenderingIntent,
    stream::Stream,
    Resolve,
};

use super::OpenPrepressInterface;

#[derive(Debug)]
pub struct ImageXObject {
    /// The width of the image, in samples
    pub width: u32,

    /// The height of the image, in samples
    pub height: u32,

    pub stream: Stream,

    /// The colour space in which image samples shall be specified; it can be
    /// any type of colour space except Pattern.
    ///
    /// If the image uses the JPXDecode filter, this entry may be present:
    ///   * If ColorSpace is present, any colour space specifications in the
    ///      JPEG2000 data shall be ignored.
    ///   * If ColorSpace is absent, the colour space specifications in the
    ///      JPEG2000 data shall be used. The Decode array shall also be
    ///      ignored unless ImageMask is true
    // todo: type of this
    color_space: Option<Object>,

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
    /// If the image stream uses the JPXDecode filter, this entry is optional and
    /// shall be ignored if present. The bit depth is determined by the conforming
    /// reader in the process of decoding the JPEG2000 image
    bits_per_component: Option<BitsPerComponent>,

    /// The name of a colour rendering intent to be used in rendering the image
    ///
    /// Default value: the current rendering intent in the graphics state
    intent: Option<RenderingIntent>,

    /// A flag indicating whether the image shall be treated as an image mask
    ///
    /// If this flag is true, the value of BitsPerComponent shall be 1 and Mask
    /// and ColorSpace shall not be specified; unmasked areas shall be painted
    /// using the current nonstroking colour
    ///
    /// Default value: false
    image_mask: bool,

    /// An image XObject defining an image mask to be applied to this image, or an
    /// array specifying a range of colours to be applied to it as a colour key
    /// mask. If ImageMask is true, this entry shall not be present
    mask: Option<ImageMask>,

    /// An array of numbers describing how to map image samples into the range of
    /// values appropriate for the image's colour space. If ImageMask is true, the
    /// array shall be either [0 1] or [1 0]; otherwise, its length shall be twice
    /// the number of colour components required by ColorSpace. If the image uses
    /// the JPXDecode filter and ImageMask is false, Decode shall be ignored by a
    /// conforming reader
    decode: Option<Vec<f32>>,

    /// A flag indicating whether image interpolation shall be performed by a conforming
    /// reader
    ///
    /// Default value: false
    interpolate: bool,

    /// An array of alternate image dictionaries for this image. The order of elements
    /// within the array shall have no significance. This entry shall not be present
    /// in an image XObject that is itself an alternate image
    alternates: Option<Vec<AlternateImage>>,

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
    s_mask: Option<SoftMask>,

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
    s_mask_in_data: i32,

    /// The name by which this image XObject is referenced in the XObject subdictionary of the
    /// current resource dictionary.
    ///
    /// This entry is obsolescent and shall no longer be used
    name: Option<String>,

    /// The integer key of the image's entry in the structural parent tree
    struct_parent: Option<i32>,

    /// The digital identifier of the image's parent Web Capture content set
    id: Option<String>,

    /// An OPI version dictionary for the image. If ImageMask is true, this entry shall be ignored
    opi: Option<OpenPrepressInterface>,

    /// A metadata stream containing metadata for the image
    metadata: Option<MetadataStream>,

    /// An optional content group or optional content membership dictionary, specifying the optional
    /// content properties for this image XObject. Before the image is processed by a conforming reader,
    /// its visibility shall be determined based on this entry. If it is determined to be invisible,
    /// the entire image shall be skipped, as if there were no Do operator to invoke it
    oc: Option<OptionalContent>,
}

impl ImageXObject {
    pub fn from_stream(mut stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let width = dict.expect_unsigned_integer("Width", resolver)?;
        let height = dict.expect_unsigned_integer("Height", resolver)?;

        let color_space = dict.get_object("ColorSpace", resolver)?;
        let bits_per_component = dict
            .get_integer("BitsPerComponent", resolver)?
            .map(BitsPerComponent::from_integer)
            .transpose()?;
        let intent = dict
            .get_name("Intent", resolver)?
            .as_deref()
            .map(RenderingIntent::from_str)
            .transpose()?;

        let image_mask = dict.get_bool("ImageMask", resolver)?.unwrap_or(false);
        let mask = dict
            .get_object("Mask", resolver)?
            .map(|obj| ImageMask::from_obj(obj, resolver))
            .transpose()?;
        let decode = dict
            .get_arr("Decode", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?;

        let interpolate = dict.get_bool("Interpolate", resolver)?.unwrap_or(false);
        let alternates = dict
            .get_arr("Alternates", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| AlternateImage::from_stream(resolver.assert_stream(obj)?, resolver))
                    .collect::<PdfResult<Vec<AlternateImage>>>()
            })
            .transpose()?;

        let s_mask = dict
            .get_stream("SMask", resolver)?
            .map(|stream| SoftMask::from_stream(stream, resolver))
            .transpose()?;
        let s_mask_in_data = dict.get_integer("SMaskInData", resolver)?.unwrap_or(0);
        let name = dict.get_name("Name", resolver)?;
        let struct_parent = dict.get_integer("StructParent", resolver)?;
        let id = dict.get_string("ID", resolver)?;
        let opi = dict
            .get_dict("OPI", resolver)?
            .map(|dict| OpenPrepressInterface::from_dict(dict, resolver))
            .transpose()?;
        let metadata = dict
            .get_stream("Metadata", resolver)?
            .map(|stream| MetadataStream::from_stream(stream, resolver))
            .transpose()?;

        let oc = dict
            .get_dict("OC", resolver)?
            .map(|dict| OptionalContent::from_dict(dict, resolver))
            .transpose()?;

        Ok(Self {
            width,
            height,
            stream,
            color_space,
            bits_per_component,
            intent,
            image_mask,
            mask,
            decode,
            interpolate,
            alternates,
            s_mask,
            s_mask_in_data,
            name,
            struct_parent,
            id,
            opi,
            metadata,
            oc,
        })
    }
}

#[derive(Debug)]
pub struct SoftMask {
    /// If a Matte entry is present, shall be the same as the Width value of the parent
    /// image; otherwise independent of it. Both images shall be mapped to the unit square
    /// in user space (as are all images), regardless of whether the samples coincide individually.
    width: u32,

    /// Same considerations as for Width.
    height: u32,

    /// Required; shall be DeviceGray.
    // todo: color space
    color_space: Object,

    bits_per_component: BitsPerComponent,

    /// Default value: [0 1]
    decode: Vec<f32>,

    interpolate: Option<bool>,

    /// An array of component values specifying the matte colour with which the image data in
    /// the parent image shall have been preblended. The array shall consist of n numbers, where
    /// n is the number of components in the colour space specified by the ColorSpace entry in
    /// the parent image's image dictionary; the numbers shall be valid colour components in that
    /// colour space. If this entry is absent, the image data shall not be preblended
    // todo: type
    matte: Option<Vec<Object>>,

    stream: Stream,
}

impl SoftMask {
    pub fn from_stream(mut stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let width = dict.expect_unsigned_integer("Width", resolver)?;
        let height = dict.expect_unsigned_integer("Height", resolver)?;
        let color_space = dict.expect_object("ColorSpace", resolver)?;
        let bits_per_component =
            BitsPerComponent::from_integer(dict.expect_integer("BitsPerComponent", resolver)?)?;
        let decode = dict
            .get_arr("Decode", resolver)?
            .map(|arr| {
                arr.into_iter()
                    .map(|obj| resolver.assert_number(obj))
                    .collect::<PdfResult<Vec<f32>>>()
            })
            .transpose()?
            .unwrap_or_else(|| vec![0.0, 1.0]);
        let interpolate = dict.get_bool("Interpolate", resolver)?;
        let matte = dict.get_arr("Matte", resolver)?;

        Ok(Self {
            width,
            height,
            color_space,
            bits_per_component,
            decode,
            interpolate,
            matte,
            stream,
        })
    }
}
#[derive(Debug)]
pub struct AlternateImage {
    /// The image XObject for the alternate image
    image: ImageXObject,

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

impl AlternateImage {
    pub fn from_stream(mut stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let default_for_printing = dict.get_bool("DefaultForPrinting", resolver)?;
        let oc = dict
            .get_dict("OC", resolver)?
            .map(|dict| OptionalContentGroup::from_dict(dict, resolver))
            .transpose()?;

        let image = ImageXObject::from_stream(stream, resolver)?;

        Ok(Self {
            image,
            default_for_printing,
            oc,
        })
    }
}

#[derive(Debug)]
pub struct OptionalContentGroup;
impl OptionalContentGroup {
    pub fn from_dict(_dict: Dictionary, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct ImageMask;

impl ImageMask {
    pub fn from_obj(_obj: Object, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}
