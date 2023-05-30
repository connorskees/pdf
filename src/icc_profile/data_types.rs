use fixed::types::extra::{U1, U15, U8};

pub(super) type F15Dot16 = fixed::FixedI32<U15>;
pub(super) type F16Dot16 = fixed::FixedU32<U15>;
pub(super) type F1Dot15 = fixed::FixedI16<U1>;
pub(super) type F8Dot8 = fixed::FixedU16<U8>;

#[derive(Debug)]
pub(super) struct Response16Number {
    interval_value: u16,
    measurement_value: F15Dot16,
}

#[derive(Debug)]
pub(super) struct XyzNumber {
    pub cie_x: F15Dot16,
    pub cie_y: F15Dot16,
    pub cie_z: F15Dot16,
}
