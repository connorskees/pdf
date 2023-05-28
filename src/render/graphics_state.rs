use crate::{
    color::ColorSpace,
    data_structures::Matrix,
    function::{Function, TransferFunction},
    halftones::Halftones,
    resources::graphics_state_parameters::{
        BlendMode, LineCapStyle, LineDashPattern, LineJoinStyle, RenderingIntent, SoftMask,
    },
};

#[derive(Debug, Default, Clone)]
pub(crate) struct GraphicsState<'a> {
    pub device_independent: DeviceIndependentGraphicsState<'a>,
    pub device_dependent: DeviceDependentGraphicsState<'a>,
}

#[derive(Debug, Clone)]
pub struct GraphicsStateColorSpace<'a> {
    pub stroking: ColorSpace<'a>,
    pub nonstroking: ColorSpace<'a>,
}

impl<'a> Default for GraphicsStateColorSpace<'a> {
    fn default() -> Self {
        Self {
            stroking: ColorSpace::DeviceGray(0.0),
            nonstroking: ColorSpace::DeviceGray(0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceIndependentGraphicsState<'a> {
    /// The current transformation matrix, which maps positions from user
    /// coordinates to device coordinates. This matrix is modified by each
    /// application of the coordinate transformation operator, cm.
    ///
    /// Initial value: a matrix that transforms default user coordinates
    /// to device coordinates
    pub current_transformation_matrix: Matrix,

    /// The current clipping path, which defines the boundary against which
    /// all output shall be cropped.
    ///
    /// Initial value: the boundary of the entire imageable portion of the
    /// output page.
    clipping_path: ClippingPath,

    /// The current colour space in which colour values shall be interpreted.
    /// There are two separate colour space parameters: one for stroking and
    /// one for all other painting operations.
    ///
    /// Initial value: DeviceGray.
    ///
    /// This library stores color values alongside their color space, and so
    /// the following documentation also applies:
    ///
    /// The current colour to be used during painting operations. The type and
    /// interpretation of this parameter depend on the current colour space;
    /// for most colour spaces, a colour value consists of one to four numbers.
    /// There are two separate colour parameters: one for stroking and one for
    /// all other painting operations.
    ///
    /// Initial value: black.
    pub color_space: GraphicsStateColorSpace<'a>,

    /// The thickness, in user space units, of paths to be stroked
    ///
    /// Initial value: 1.0.
    pub line_width: f32,

    /// A code specifying the shape of the endpoints for any open path that is
    /// stroked
    ///
    /// Initial value: 0, for square butt caps.
    pub line_cap_style: LineCapStyle,

    /// A code specifying the shape of joints between connected segments of a
    /// stroked path
    ///
    /// Initial value: 0, for mitered joins.
    pub line_join_style: LineJoinStyle,

    /// The maximum length of mitered line joins for stroked paths. This
    /// parameter limits the length of “spikes” produced when line segments
    /// join at sharp angles.
    ///
    /// Initial value: 10.0, for a miter cutoff below approximately 11.5
    /// degrees
    pub miter_limit: f32,

    /// A description of the dash pattern to be used when paths are stroked.
    ///
    /// Initial value: a solid line.
    pub line_dash_pattern: LineDashPattern,

    /// The rendering intent to be used when converting CIE-based colours to
    /// device colours.
    ///
    /// Initial value: RelativeColorimetric.
    pub rendering_intent: RenderingIntent,

    /// A flag specifying whether to compensate for possible rasterization
    /// effects when stroking a path with a line width that is small relative
    /// to the pixel resolution of the output device.
    ///
    /// NOTE This is considered a device-independent parameter, even though the
    /// details of its effects are device-dependent.
    ///
    /// Initial value: false.
    pub stroke_adjustment: bool,

    /// The current blend mode to be used in the transparent imaging model. A
    /// conforming reader shall implicitly reset this parameter to its initial
    /// value at the beginning of execution of a transparency group XObject
    ///
    /// Initial value: Normal.
    pub blend_mode: BlendMode,

    /// A soft-mask dictionary specifying the mask shape or mask opacity values
    /// to be used in the transparent imaging model, or the name None if no such
    /// mask is specified. A conforming reader shall implicitly reset this
    /// parameter implicitly reset to its initial value at the beginning of
    /// execution of a transparency group XObject
    ///
    /// Initial value: None.
    pub soft_mask: SoftMask<'a>,

    /// The constant shape or constant opacity value to be used in the transparent
    /// imaging model. There are two separate alpha constant parameters: one for
    /// stroking and one for all other painting operations. A conforming reader
    /// shall implicitly reset this parameter to its initial value at the beginning
    /// of execution of a transparency group XObject
    ///
    /// Initial value: 1.0.
    pub stroking_alpha_constant: f32,
    pub nonstroking_alpha_constant: f32,

    /// A flag specifying whether the current soft mask and alpha constant
    /// parameters shall be interpreted as shape values (true) or opacity values
    /// (false). This flag also governs the interpretation of the SMask entry,
    /// if any, in an image dictionary
    ///
    /// Initial value: false.
    pub alpha_source: bool,
}

impl Default for DeviceIndependentGraphicsState<'_> {
    fn default() -> Self {
        Self {
            current_transformation_matrix: Matrix::identity(),
            clipping_path: ClippingPath,
            color_space: GraphicsStateColorSpace::default(),
            line_width: 1.0,
            line_cap_style: LineCapStyle::Butt,
            line_join_style: LineJoinStyle::Miter,
            miter_limit: 10.0,
            line_dash_pattern: LineDashPattern::solid(),
            rendering_intent: RenderingIntent::RelativeColorimetric,
            stroke_adjustment: false,
            blend_mode: BlendMode::Normal,
            soft_mask: SoftMask::None,
            stroking_alpha_constant: 1.0,
            nonstroking_alpha_constant: 1.0,
            alpha_source: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceDependentGraphicsState<'a> {
    /// A flag specifying (on output devices that support the overprint control
    /// feature) whether painting in one set of colorants should cause the
    /// corresponding areas of other colorants to be erased (false) or left
    /// unchanged (true). In PDF 1.3, there are two separate overprint parameters:
    /// one for stroking and one for all other painting operations.
    ///
    /// Initial value: false.
    pub should_overprint: bool,

    pub should_overprint_stroking: bool,

    /// A code specifying whether a colour component value of 0 in a DeviceCMYK
    /// colour space should erase that component (0) or leave it unchanged (1) when
    /// overprinting
    ///
    /// Initial value: 0.
    pub overprint_mode: i32,

    /// A function that calculates the level of the black colour component to
    /// use when converting RGB colours to CMYK
    ///
    /// Initial value: a conforming reader shall initialize this to a suitable
    /// device dependent value.
    // todo: this is temporarily nullable, as it's unclear what the default fn
    // should be
    pub black_generation: Option<Function<'a>>,

    /// A function that calculates the reduction in the levels of the cyan,
    /// magenta, and yellow colour components to compensate for the amount of
    /// black added by black generation
    ///
    /// Initial value: a conforming reader shall initialize this to a suitable
    /// device dependent value.
    // todo: this is temporarily nullable, as it's unclear what the default fn
    // should be
    pub undercolor_removal: Option<Function<'a>>,

    /// A function that adjusts device gray or colour component levels to
    /// compensate for nonlinear response in a particular output device
    ///
    /// Initial value: a conforming reader shall initialize this to a suitable
    /// device dependent value.
    pub transfer: TransferFunction<'a>,

    /// A halftone screen for gray and colour rendering, specified as a halftone
    /// dictionary or stream
    pub halftones: Halftones<'a>,

    /// The precision with which curves shall be rendered on the output device.
    /// The value of this parameter (positive number) gives the maximum error
    /// tolerance, measured in output device pixels; smaller numbers give smoother
    /// curves at the expense of more computation and memory use.
    ///
    /// Initial value: 1.0.
    pub flatness_tolerance: f32,

    /// The precision with which colour gradients are to be rendered on the
    /// output device. The value of this parameter (0 to 1.0) gives the maximum
    /// error tolerance, expressed as a fraction of the range of each colour
    /// component; smaller numbers give smoother colour transitions at the
    /// expense of more computation and memory use.
    ///
    /// Initial value: a conforming reader shall initialize this to a suitable
    /// device dependent value.
    pub smoothness_tolerance: f32,
}

impl Default for DeviceDependentGraphicsState<'_> {
    fn default() -> Self {
        Self {
            should_overprint: false,
            should_overprint_stroking: false,
            overprint_mode: 0,
            black_generation: None,
            undercolor_removal: None,
            transfer: TransferFunction::Identity,
            halftones: Halftones::Default,
            flatness_tolerance: 1.0,
            smoothness_tolerance: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
struct ClippingPath;
