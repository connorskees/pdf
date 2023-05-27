use super::{EF2Dot14, F26Dot6};

#[derive(Debug)]
pub struct TrueTypeGraphicsState {
    pub auto_flip: bool,
    pub control_value_cut_in: F26Dot6,
    pub delta_base: u32,
    pub delta_shift: u32,
    pub dual_projection_vector: Option<Vector>,
    pub freedom_vector: Vector,
    pub instruct_control: (),
    pub loop_counter: u32,
    pub minimum_distance: F26Dot6,
    pub projection_vector: Vector,
    pub round_state: RoundState,
    pub rp0: u32,
    pub rp1: u32,
    pub rp2: u32,
    pub scan_control: (),
    pub single_width_cut_in: F26Dot6,
    pub single_width_value: F26Dot6,
    pub zp0: u32,
    pub zp1: u32,
    pub zp2: u32,
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
            instruct_control: (),
            loop_counter: 1,
            minimum_distance: F26Dot6::ONE,
            projection_vector: Vector::x_axis(),
            round_state: RoundState::ToGrid,
            rp0: 0,
            rp1: 0,
            rp2: 0,
            scan_control: (),
            single_width_cut_in: F26Dot6::ZERO,
            single_width_value: F26Dot6::ZERO,
            zp0: 1,
            zp1: 1,
            zp2: 1,
        }
    }
}
