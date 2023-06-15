use super::consts::{EXPERT_ENCODING, STANDARD_ENCODING};

#[derive(Debug)]
pub enum CffEncoding<'a> {
    Standard,
    Expert,
    Zero { n_codes: u8, codes: &'a [u8] },
    One(Vec<EncodingRangeOne>),
}

#[derive(Debug)]
pub struct EncodingRangeOne {
    pub first: u8,
    /// Number of glyphs after `first`
    pub count: u8,
}

impl<'a> CffEncoding<'a> {
    pub fn lookup(&self, charcode: u32) -> Option<u16> {
        match self {
            Self::Standard => {
                let (idx, _) = STANDARD_ENCODING
                    .iter()
                    .copied()
                    .enumerate()
                    .find(|(_, code)| *code as u32 == charcode)?;

                Some(idx as u16)
            }
            Self::Expert => {
                let (idx, _) = EXPERT_ENCODING
                    .iter()
                    .copied()
                    .enumerate()
                    .find(|(_, code)| *code as u32 == charcode)?;

                Some(idx as u16)
            }
            CffEncoding::Zero { codes, .. } => {
                let (idx, _) = codes
                    .iter()
                    .copied()
                    .enumerate()
                    .find(|(_, code)| *code as u32 == charcode)?;

                Some(idx as u16)
            }
            CffEncoding::One(ranges) => {
                let mut idx = 0;
                for range in ranges {
                    if charcode > idx + range.count as u32 + 1 {
                        idx += range.count as u32 + 1;
                    } else {
                        let offset = charcode - idx - range.count as u32 - 1;

                        if offset == 0 {
                            return Some(range.first as u16);
                        } else {
                            return Some(range.first as u16 + offset as u16);
                        }
                    }
                }

                None
            }
        }
    }
}
