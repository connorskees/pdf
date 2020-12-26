use crate::{
    error::{ParseError, PdfResult},
    NUMBERS,
};

#[derive(Debug)]
pub struct Date {
    year: Option<u16>,
    month: Option<u16>,
    day: Option<u16>,
    hour: Option<u16>,
    minute: Option<u16>,
    second: Option<u16>,

    ut_relationship: Option<UtRelationship>,
    ut_hour_offset: Option<u16>,
    ut_minute_offset: Option<u16>,
}

impl Date {
    pub(crate) fn from_str(s: &str) -> PdfResult<Self> {
        let mut chars = s.bytes();

        let mut date = Date {
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            ut_relationship: None,
            ut_hour_offset: None,
            ut_minute_offset: None,
        };

        match chars.next() {
            Some(b'D') => {}
            found => {
                return Err(ParseError::MismatchedByte {
                    expected: b'D',
                    found,
                });
            }
        }

        match chars.next() {
            Some(b':') => {}
            found => {
                return Err(ParseError::MismatchedByte {
                    expected: b':',
                    found,
                });
            }
        }

        macro_rules! unit {
            ($unit:ident, $len:literal) => {
                let mut $unit = 0;

                for _ in 0..$len {
                    let next = match chars.next() {
                        Some(n @ b'0'..=b'9') => n - b'0',
                        None => return Ok(date),
                        found => {
                            return Err(ParseError::MismatchedByteMany {
                                expected: NUMBERS,
                                found,
                            });
                        }
                    };

                    $unit *= 10;
                    $unit += next as u16;
                }

                date.$unit = Some($unit);
            };
        }

        unit!(year, 4);
        unit!(month, 2);
        unit!(day, 2);
        unit!(hour, 2);
        unit!(minute, 2);
        unit!(second, 2);
        date.ut_relationship = chars.next().map(UtRelationship::from_byte).transpose()?;
        unit!(ut_hour_offset, 2);
        match chars.next() {
            Some(b'\'') => {}
            found => {
                return Err(ParseError::MismatchedByte {
                    expected: b'\'',
                    found,
                })
            }
        }
        unit!(ut_minute_offset, 2);

        Ok(date)
    }
}

#[derive(Debug)]
enum UtRelationship {
    Plus,
    Minus,
    Equal,
}

impl UtRelationship {
    pub fn from_byte(b: u8) -> PdfResult<Self> {
        Ok(match b {
            b'+' => Self::Plus,
            b'-' => Self::Minus,
            b'Z' => Self::Equal,
            found => {
                return Err(ParseError::MismatchedByteMany {
                    expected: &[b'+', b'-', b'Z'],
                    found: Some(found),
                })
            }
        })
    }
}
