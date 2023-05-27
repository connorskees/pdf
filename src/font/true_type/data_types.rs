use fixed::types::extra::{U2, U26};

/// 16-bit signed fraction
#[derive(Debug)]
pub struct ShortFraction(pub i16);

/// 16.16-bit signed fixed-point number
#[derive(Debug)]
pub struct Fixed(pub i32);

#[derive(Debug)]
pub enum DataType {
    /// 16-bit signed fraction
    ShortFraction(ShortFraction),

    /// 16.16-bit signed fixed-point number
    Fixed(i32),

    /// 16-bit signed integer that describes a quantity in FUnits, the smallest
    /// measurable distance in em space
    FWord(FWord),

    /// 16-bit unsigned integer that describes a quantity in FUnits, the smallest
    /// measurable distance in em space
    UnsignedFWord(u16),

    /// 16-bit signed fixed number with the low 14 bits representing fraction.
    F2Dot14(i16),

    /// The long internal format of a date in seconds since 12:00 midnight, January
    /// 1, 1904. It is represented as a signed 64-bit integer
    LongDateTime(LongDateTime),
}

#[derive(Debug)]
pub struct LongDateTime(pub i64);
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct FWord(pub i16);

/// sign extended 8-bit interger
pub type Eint8 = i8;
/// zero extended 16-bit unsigned integer
pub type Euint16 = u16;
/// sign extended 16-bit signed integer that describes a quanity in FUnits, the
/// smallest measurable unit in the em space
pub type EFWord = i16;
/// sign extended 16-bit signed fixed number with the low 14 bits representing fraction
pub type EF2Dot14 = fixed::FixedI16<U2>;
/// 32-bit unsigned integer
pub type UInt32 = u32;
/// 32-bit signed interger
pub type Int32 = i32;
/// 32-bit signed fixed number with the low 6 bits representing fraction
pub type F26Dot6 = fixed::FixedI32<U26>;
/// any 32 bit quantity
pub type StkElt = u32;
