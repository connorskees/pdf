use crate::{
    data_structures::{Matrix, Rectangle},
    error::PdfResult,
    objects::{Dictionary, Object},
    pdf_enum,
    shading::ShadingObject,
    stream::Stream,
    Resolve,
};

use super::{graphics_state_parameters::GraphicsStateParameters, Resources};

#[derive(Debug)]
pub enum Pattern {
    /// Tiling patterns consist of a small graphical figure (called a pattern cell) that is
    /// replicated at fixed horizontal and vertical intervals to fill the area to be painted.
    /// The graphics objects to use for tiling shall be described by a content stream.
    Tiling(TilingPattern),

    /// Shading patterns define a gradient fill that produces a smooth transition between
    /// colours across the area. The colour to use shall be specified as a function of position
    /// using any of a variety of methods.
    Shading(ShadingPattern),
}

impl Pattern {
    const TYPE: &'static str = "Pattern";

    pub fn from_object(obj: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let obj = resolver.resolve(obj)?;

        Ok(
            if let Ok(mut stream) = resolver.assert_stream(obj.clone()) {
                let dict = &mut stream.dict.other;
                dict.expect_type(Self::TYPE, resolver, false)?;

                let pattern_type =
                    PatternType::from_integer(dict.expect_integer("PatternType", resolver)?)?;

                assert_eq!(pattern_type, PatternType::Tiling);

                Pattern::Tiling(TilingPattern::from_stream(stream, resolver)?)
            } else {
                let mut dict = resolver.assert_dict(obj)?;
                dict.expect_type(Self::TYPE, resolver, false)?;

                let pattern_type =
                    PatternType::from_integer(dict.expect_integer("PatternType", resolver)?)?;

                assert_eq!(pattern_type, PatternType::Shading);

                Pattern::Shading(ShadingPattern::from_dict(dict, resolver)?)
            },
        )
    }
}

#[derive(Debug)]
pub struct TilingPattern {
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
    resources: Resources,

    /// An array of six numbers specifying the pattern matrix.
    ///
    /// Default value: the identity matrix [1 0 0 1 0 0].
    matrix: Matrix,
}

impl TilingPattern {
    pub fn from_stream(stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let mut dict = stream.dict.other;
        let paint_type = PaintType::from_integer(dict.expect_integer("PaintType", resolver)?)?;
        let tiling_type = TilingType::from_integer(dict.expect_integer("TilingType", resolver)?)?;
        let bbox = dict.expect_rectangle("BBox", resolver)?;
        let x_step = dict.expect_number("XStep", resolver)?;
        let y_step = dict.expect_number("YStep", resolver)?;
        let resources = Resources::from_dict(dict.expect_dict("Resources", resolver)?, resolver)?;
        let matrix = dict
            .get_matrix("Matrix", resolver)?
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

#[derive(Debug)]
pub struct ShadingPattern {
    /// A shading object defining the shading pattern's gradient fill
    shading: ShadingObject,

    /// An array of six numbers specifying the pattern matrix
    ///
    /// Default value: the identity matrix [1 0 0 1 0 0].
    matrix: Matrix,

    /// A graphics state parameter dictionary containing graphics state parameters to be put
    /// into effect temporarily while the shading pattern is painted. Any parameters that are
    /// so specified shall be inherited from the graphics state that was in effect at the
    /// beginning of the content stream in which the pattern is defined as a resource
    ext_g_state: Option<GraphicsStateParameters>,
}

impl ShadingPattern {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let shading = ShadingObject::from_obj(dict.expect_object("Shading", resolver)?, resolver)?;

        let matrix = dict
            .get_matrix("Matrix", resolver)?
            .unwrap_or_else(Matrix::identity);

        let ext_g_state = dict
            .get_dict("ExtGState", resolver)?
            .map(|dict| GraphicsStateParameters::from_dict(dict, resolver))
            .transpose()?;

        Ok(Self {
            shading,
            matrix,
            ext_g_state,
        })
    }
}

pdf_enum!(
    int
    #[derive(Debug, PartialEq, Eq)]
    enum PatternType {
        Tiling = 1,
        Shading = 2,
    }
);

pdf_enum!(
    int
    #[derive(Debug)]
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
);

pdf_enum!(
    int
    #[derive(Debug)]
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
);
