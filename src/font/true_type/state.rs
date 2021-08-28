use crate::geometry::Line;

struct TrueTypeGraphicsState {
    /// Controls whether the sign of control value table entries will be changed
    /// to match the sign of the actual distance measurement with which it is
    /// compared. Setting auto flip to TRUE makes it possible to control distances
    /// measured with or against the projection vector with a single control value
    /// table entry. When auto flip is set to FALSE, distances must be measured
    /// with the projection vector.
    ///
    /// Default: true
    auto_flip: bool,

    /// Limits the regularizing effects of control value table entries to cases
    /// where the difference between the table value and the measurement taken
    /// from the original outline is sufficiently small
    ///
    /// Default: 17/16 pixels
    control_value_cut_in: f32,

    /// Establishes the base value used to calculate the range of point sizes to
    /// which a given DELTAC[] or DELTAP[] instruction will apply. The formulas
    /// given below are used to calculate the range of the various DELTA instructions
    ///
    /// Default: 9
    delta_base: f32,

    /// Determines the range of movement and smallest magnitude of movement (the
    /// step) in a DELTAC[] or DELTAP[] instruction. Changing the value of the
    /// delta shift makes it possible to trade off fine control of point movement
    /// for range of movement. A low delta shift favors range of movement over
    /// fine control. A high delta shift favors fine control over range of movement.
    /// The step has the value 1/2 to the power delta shift. The range of movement
    /// is calculated by taking the number of steps allowed (16) and multiplying
    /// it by the step
    ///
    /// The legal range for delta shift is zero through six. Negative values are illegal
    ///
    /// Default: 3
    delta_shift: f32,

    /// A second projection vector set to a line defined by the original outline
    /// location of two points. The dual projection vector is used when it is
    /// necessary to measure distances from the scaled outline before any instructions
    /// were executed
    ///
    /// Default: None
    dual_projection_vector: Option<Line>,

    /// A unit vector that establishes an axis along which points can move
    ///
    /// Default: x-axis
    freedom_vector: Line,

    /// Makes it possible to turn off instructions under some circumstances.
    /// When set to TRUE, no instructions will be executed
    ///
    /// Default: false
    instruct_control: bool,

    /// Makes it possible to repeat certain instructions a designated number of
    /// times. The default value of one assures that unless the value of loop
    /// is altered, these instructions will execute one time
    ///
    /// Default: 1
    loop_count: f32,

    /// Establishes the smallest possible value to which a distance will be rounded
    ///
    /// Default: 1 pixel
    minimum_distance: f32,

    /// A unit vector whose direction establishes an axis along which distances
    /// are measured
    ///
    /// Default: x-axis
    projection_vector: Line,

    /// Determines the manner in which values are rounded. Can be set to a number
    /// of predefined states or to a customized state with the SROUND or S45ROUND
    /// instructions
    ///
    /// Default: 1
    round_state: f32,

    /// The first of three reference points. References a point number that together
    /// with a zone designation specifies a point in either the glyph zone or the
    /// twilight zone
    ///
    /// Default: 0
    rp0: f32,

    /// The second of three reference points. References a point number that
    /// together with a zone designation specifies a point in either the glyph
    /// zone or the twilight zone
    ///
    /// Default: 0
    rp1: f32,

    /// The third of three reference points. References a point number that
    /// together with a zone designation specifies a point in to either the glyph
    /// zone or the twilight zone
    ///
    /// Default: 0
    rp2: f32,

    /// Determines whether the interpreter will activate dropout control for the
    /// current glyph. Use of the dropout control mode can depend upon the currently
    /// prevailing combination of the following three conditions:
    ///
    ///   1. Is the glyph rotated?
    ///   2. Is the glyph stretched?
    ///   3. Is the current pixel per em setting less than a specified threshold?
    ///
    /// It is also possible to block dropout control if one of the above conditions
    /// is false
    ///
    /// Default: false
    scan_control: bool,

    /// The distance difference below which the interpreter will replace a CVT
    /// distance or an actual distance in favor of the single width value
    ///
    /// Default: 0 pixels
    single_width_cut_in: f32,

    /// The value used in place of the control value table distance or the actual
    /// distance value when the difference between that distance and the single
    /// width value is less than the single width cut-in
    ///
    /// Default: 0 pixels
    single_width_value: f32,

    /// The first of three zone pointers. Can be set to reference either the glyph
    /// zone (Z0) or the twilight zone (Z1)
    ///
    /// Default: 1
    zp0: f32,

    /// The second of three zone pointers. Can be set to reference either the
    /// twilight zone (Z0) or the glyph zone (Z1)
    ///
    /// Default: 1
    zp1: f32,

    /// The third of three zone pointers. Can be set to reference either the
    /// twilight zone (Z0) or the glyph zone (Z1)
    ///
    /// Default: 1
    zp2: f32,
}

impl Default for TrueTypeGraphicsState {
    fn default() -> Self {
        Self {
            auto_flip: true,
            control_value_cut_in: 17.0 / 16.0,
            delta_base: 9.0,
            delta_shift: 3.0,
            dual_projection_vector: None,
            freedom_vector: Line::x_axis(),
            instruct_control: false,
            loop_count: 1.0,
            minimum_distance: 1.0,
            projection_vector: Line::x_axis(),
            round_state: 1.0,
            rp0: 0.0,
            rp1: 0.0,
            rp2: 0.0,
            scan_control: false,
            single_width_cut_in: 0.0,
            single_width_value: 0.0,
            zp0: 0.0,
            zp1: 0.0,
            zp2: 0.0,
        }
    }
}
