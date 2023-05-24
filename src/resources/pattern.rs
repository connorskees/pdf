use crate::{
    data_structures::{Matrix, Rectangle},
    error::PdfResult,
    objects::Object,
    shading::ShadingObject,
    stream::Stream,
    FromObj, Resolve,
};

use super::{graphics_state_parameters::GraphicsStateParameters, Resources};

#[derive(Debug, Clone)]
pub enum Pattern<'a> {
    /// Tiling patterns consist of a small graphical figure (called a pattern cell) that is
    /// replicated at fixed horizontal and vertical intervals to fill the area to be painted.
    /// The graphics objects to use for tiling shall be described by a content stream.
    Tiling(TilingPattern<'a>),

    /// Shading patterns define a gradient fill that produces a smooth transition between
    /// colours across the area. The colour to use shall be specified as a function of position
    /// using any of a variety of methods.
    Shading(ShadingPattern<'a>),
}

impl<'a> Pattern<'a> {
    const TYPE: &'static str = "Pattern";
}

impl<'a> FromObj<'a> for Pattern<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        Ok(
            if let Ok(mut stream) = resolver.assert_stream(obj.clone()) {
                let dict = &mut stream.dict.other;
                dict.expect_type(Self::TYPE, resolver, false)?;

                let pattern_type = dict.expect::<PatternType>("PatternType", resolver)?;

                assert_eq!(pattern_type, PatternType::Tiling);

                Pattern::Tiling(TilingPattern::from_stream(stream, resolver)?)
            } else {
                let mut dict = resolver.assert_dict(obj)?;
                dict.expect_type(Self::TYPE, resolver, false)?;

                let pattern_type = dict.expect::<PatternType>("PatternType", resolver)?;

                assert_eq!(pattern_type, PatternType::Shading);

                Pattern::Shading(ShadingPattern::from_obj(
                    Object::Dictionary(dict),
                    resolver,
                )?)
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct TilingPattern<'a> {
    /// A code that determines how the colour of the pattern cell shall be specified
    paint_type: PaintType,

    /// A code that controls adjustments to the spacing of tiles relative to the device pixel grid
    tiling_type: TilingType,

    /// An array of four numbers in the pattern coordinate system giving the coordinates of the
    /// left, bottom, right, and top edges, respectively, of the pattern cell's bounding box. These
    /// boundaries shall be used to clip the pattern cell
    bbox: Rectangle,

    /// The desired horizontal spacing between pattern cells, measured in the pattern coordinate system
    x_step: f32,

    /// The desired vertical spacing between pattern cells, measured in the pattern coordinate system
    ///
    /// XStep and YStep may differ from the dimensions of the pattern cell implied by the BBox entry.
    /// This allows tiling with irregularly shaped figures
    ///
    /// XStep and YStep may be either positive or negative but shall not be zero
    y_step: f32,

    /// A resource dictionary that shall contain all of the named resources required by the pattern's
    /// content stream
    resources: Resources<'a>,

    /// An array of six numbers specifying the pattern matrix.
    ///
    /// Default value: the identity matrix [1 0 0 1 0 0].
    matrix: Matrix,
}

impl<'a> TilingPattern<'a> {
    pub fn from_stream(stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut dict = stream.dict.other;
        let paint_type = PaintType::from_integer(dict.expect_integer("PaintType", resolver)?)?;
        let tiling_type = TilingType::from_integer(dict.expect_integer("TilingType", resolver)?)?;
        let bbox = dict.expect::<Rectangle>("BBox", resolver)?;
        let x_step = dict.expect_number("XStep", resolver)?;
        let y_step = dict.expect_number("YStep", resolver)?;
        let resources = Resources::from_dict(dict.expect_dict("Resources", resolver)?, resolver)?;
        let matrix = dict
            .get::<Matrix>("Matrix", resolver)?
            .unwrap_or_else(Matrix::identity);

        Ok(Self {
            paint_type,
            tiling_type,
            bbox,
            x_step,
            y_step,
            resources,
            matrix,
        })
    }
}

#[derive(Debug, Clone, FromObj)]
pub struct ShadingPattern<'a> {
    /// A shading object defining the shading pattern's gradient fill
    #[field("Shading")]
    shading: ShadingObject<'a>,

    /// An array of six numbers specifying the pattern matrix
    ///
    /// Default value: the identity matrix [1 0 0 1 0 0].
    #[field("Matrix")]
    matrix: Matrix,

    /// A graphics state parameter dictionary containing graphics state parameters to be put
    /// into effect temporarily while the shading pattern is painted. Any parameters that are
    /// so specified shall be inherited from the graphics state that was in effect at the
    /// beginning of the content stream in which the pattern is defined as a resource
    #[field("ExtGState")]
    ext_g_state: Option<GraphicsStateParameters<'a>>,
}

#[pdf_enum(Integer)]
enum PatternType {
    Tiling = 1,
    Shading = 2,
}

#[pdf_enum(Integer)]
enum PaintType {
    /// The pattern's content stream shall specify the colours used to paint the pattern
    /// cell. When the content stream begins execution, the current colour is the one
    /// that was initially in effect in the pattern's parent content stream. This is
    /// similar to the definition of the pattern matrix
    Colored = 1,

    /// The pattern's content stream shall not specify any colour information. Instead,
    /// the entire pattern cell is painted with a separately specified colour each time
    /// the pattern is used. Essentially, the content stream describes a stencil
    /// through which the current colour shall be poured. The content stream shall not
    /// invoke operators that specify colours or other colourrelated parameters in the
    /// graphics state; otherwise, an error occurs. The content stream may paint an
    /// image mask, however, since it does not specify any colour information
    Uncolored = 2,
}

#[pdf_enum(Integer)]
enum TilingType {
    /// Pattern cells shall be spaced consistently -- that is, by a multiple of a device
    /// pixel. To achieve this, the conforming reader may need to distort the pattern
    /// cell slightly by making small adjustments to XStep, YStep, and the transformation
    /// matrix. The amount of distortion shall not exceed 1 device pixel
    ConstantSpacing = 1,

    /// The pattern cell shall not be distorted, but the spacing between pattern cells
    /// may vary by as much as 1 device pixel, both horizontally and vertically, when
    /// the pattern is painted. This achieves the spacing requested by XStep and YStep
    /// on average but not necessarily for each individual pattern cell
    NoDistortion = 2,

    /// Pattern cells shall be spaced consistently as in tiling type 1 but with additional
    /// distortion permitted to enable a more efficient implementation
    ConstantSpacingAndFasterTiling = 3,
}
