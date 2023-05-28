use super::{EF2Dot14, F26Dot6};

#[derive(Debug)]
pub struct TrueTypeGraphicsState {
    /// Controls whether the sign of control value table entries will be changed
    /// to match the sign of the actual distance measurement with which it is
    /// compared. Setting auto flip to TRUE makes it possible to control distances
    /// measured with or against the projection vector with a single control value
    /// table entry. When auto flip is set to FALSE, distances must be measured
    /// with the projection vector.
    ///
    /// Default: true
    pub auto_flip: bool,

    /// Limits the regularizing effects of control value table entries to cases
    /// where the difference between the table value and the measurement taken
    /// from the original outline is sufficiently small
    ///
    /// Default: 17/16 pixels
    pub control_value_cut_in: F26Dot6,

    /// Establishes the base value used to calculate the range of point sizes to
    /// which a given DELTAC[] or DELTAP[] instruction will apply. The formulas
    /// given below are used to calculate the range of the various DELTA instructions
    ///
    /// Default: 9
    pub delta_base: u32,

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
    pub delta_shift: u32,

    /// A second projection vector set to a line defined by the original outline
    /// location of two points. The dual projection vector is used when it is
    /// necessary to measure distances from the scaled outline before any instructions
    /// were executed
    ///
    /// Default: None
    pub dual_projection_vector: Option<Vector>,

    /// A unit vector that establishes an axis along which points can move
    ///
    /// Default: x-axis
    pub freedom_vector: Vector,

    /// Makes it possible to turn off instructions under some circumstances.
    /// When set to TRUE, no instructions will be executed
    ///
    /// Default: false
    pub instruct_control: bool,

    /// Makes it possible to repeat certain instructions a designated number of
    /// times. The default value of one assures that unless the value of loop
    /// is altered, these instructions will execute one time
    ///
    /// Default: 1
    pub loop_counter: u32,

    /// Establishes the smallest possible value to which a distance will be rounded
    ///
    /// Default: 1 pixel
    pub minimum_distance: F26Dot6,

    /// A unit vector whose direction establishes an axis along which distances
    /// are measured
    ///
    /// Default: x-axis
    pub projection_vector: Vector,

    /// Determines the manner in which values are rounded. Can be set to a number
    /// of predefined states or to a customized state with the SROUND or S45ROUND
    /// instructions
    ///
    /// Default: 1 (grid)
    pub round_state: RoundState,

    /// The first of three reference points. References a point number that together
    /// with a zone designation specifies a point in either the glyph zone or the
    /// twilight zone
    ///
    /// Default: 0
    pub rp0: u32,

    /// The second of three reference points. References a point number that
    /// together with a zone designation specifies a point in either the glyph
    /// zone or the twilight zone
    ///
    /// Default: 0
    pub rp1: u32,

    /// The third of three reference points. References a point number that
    /// together with a zone designation specifies a point in to either the glyph
    /// zone or the twilight zone
    ///
    /// Default: 0
    pub rp2: u32,

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
    pub scan_control: bool,

    /// The distance difference below which the interpreter will replace a CVT
    /// distance or an actual distance in favor of the single width value
    ///
    /// Default: 0 pixels
    pub single_width_cut_in: F26Dot6,

    /// The value used in place of the control value table distance or the actual
    /// distance value when the difference between that distance and the single
    /// width value is less than the single width cut-in
    ///
    /// Default: 0 pixels
    pub single_width_value: F26Dot6,

    /// The first of three zone pointers. Can be set to reference either the glyph
    /// zone (Z0) or the twilight zone (Z1)
    ///
    /// Default: 1 (glyph)
    pub zp0: Zone,

    /// The second of three zone pointers. Can be set to reference either the
    /// twilight zone (Z0) or the glyph zone (Z1)
    ///
    /// Default: 1 (glyph)
    pub zp1: Zone,

    /// The third of three zone pointers. Can be set to reference either the
    /// twilight zone (Z0) or the glyph zone (Z1)
    ///
    /// Default: 1 (glyph)
    pub zp2: Zone,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Zone {
    Twilight = 0,
    Glyph = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RoundState {
    /// Rounds to the nearest half-number (for example, 1.45 is rounded to 1.5)
    ToHalfGrid = 0,
    /// Rounds to the nearest integer
    ToGrid = 1,
    /// Rounds to an integer or half-number, whichever is nearer
    ToDoubleGrid = 2,
    /// Always rounds to the lower integer (for example, 1.9 is rounded to 1.0)
    DownToGrid = 3,
    /// Always rounds to the higher integer (for example, 1.1 is rounded to 2.0)
    UpToGrid = 4,
    /// No Rounding
    Off = 5,
    Custom {
        /// the length of the separation or space between rounded values
        period: f32,
        /// the offset of the rounded values from multiples of the period
        phase: f32,
        /// the point at which the direction of rounding changes (0.5 by default):
        /// if the number is less than this the direction of rounding is down;
        /// if equal or greater, the direction is up
        threshold: f32,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    px: EF2Dot14,
    py: EF2Dot14,
}

impl Vector {
    pub fn x_axis() -> Self {
        Self {
            px: EF2Dot14::ONE,
            py: EF2Dot14::ZERO,
        }
    }

    pub fn y_axis() -> Self {
        Self {
            px: EF2Dot14::ZERO,
            py: EF2Dot14::ONE,
        }
    }
}

impl Default for TrueTypeGraphicsState {
    fn default() -> Self {
        Self {
            auto_flip: true,
            control_value_cut_in: F26Dot6::from_num(17) / 16,
            delta_base: 9,
            delta_shift: 3,
            dual_projection_vector: None,
            freedom_vector: Vector::x_axis(),
            instruct_control: false,
            loop_counter: 1,
            minimum_distance: F26Dot6::ONE,
            projection_vector: Vector::x_axis(),
            round_state: RoundState::ToGrid,
            rp0: 0,
            rp1: 0,
            rp2: 0,
            scan_control: false,
            single_width_cut_in: F26Dot6::ZERO,
            single_width_value: F26Dot6::ZERO,
            zp0: Zone::Glyph,
            zp1: Zone::Glyph,
            zp2: Zone::Glyph,
        }
    }
}
