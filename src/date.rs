use crate::{
    error::{ParseError, PdfResult},
    NUMBERS,
};

#[derive(Debug, PartialEq)]
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
        let mut chars = s.bytes().peekable();

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
                        found @ Some(..) => {
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
        match chars.peek().cloned() {
            Some(b'\'') => {
                chars.next();

                if !chars.peek().is_none() {
                    return Err(ParseError::MismatchedByte {
                        expected: b'\'',
                        found: chars.next(),
                    });
                }
            }
            _ => {}
        }
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
        match chars.next() {
            Some(b'\'') | None => {}
            found => {
                return Err(ParseError::MismatchedByte {
                    expected: b'\'',
                    found,
                })
            }
        }

        Ok(date)
    }
}

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod test {
    use super::{Date, UtRelationship};

    #[test]
    /// Test case taken from a Libre Office pdf
    fn ends_with_single_quote_no_ut_hour() {
        assert_eq!(
            Date {
                year: Some(2020),
                month: Some(11),
                day: Some(25),
                hour: Some(2),
                minute: Some(11),
                second: Some(08),
                ut_relationship: Some(UtRelationship::Equal),
                ut_hour_offset: None,
                ut_minute_offset: None,
            },
            Date::from_str("D:20201125021108Z'").unwrap()
        )
    }

    #[test]
    fn ut_min_and_ut_hour_set() {
        assert_eq!(
            Date {
                year: Some(2020),
                month: Some(12),
                day: Some(3),
                hour: Some(18),
                minute: Some(48),
                second: Some(27),
                ut_relationship: Some(UtRelationship::Minus),
                ut_hour_offset: Some(8),
                ut_minute_offset: Some(0),
            },
            Date::from_str("D:20201203184827-08'00'").unwrap()
        )
    }

    #[test]
    fn no_ut_set() {
        assert_eq!(
            Date {
                year: Some(2008),
                month: Some(6),
                day: Some(11),
                hour: Some(16),
                minute: Some(56),
                second: Some(3),
                ut_relationship: None,
                ut_hour_offset: None,
                ut_minute_offset: None,
            },
            Date::from_str("D:20080611165603").unwrap()
        )
    }
}
