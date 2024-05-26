use crate::{
    font::{cff::dict::CffDictInterpreter, CffFile},
    parse_binary::BinaryParser,
};

use super::{
    CffCharset, CffEncoding, CffHeader, CffIndex, CharsetRangeOne, CharsetRangeTwo,
    EncodingRangeOne,
};

pub struct CffParser<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

impl<'a> CffParser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    pub fn parse(&mut self) -> anyhow::Result<CffFile<'a>> {
        let header = self.parse_header()?;
        assert_eq!(self.cursor, header.header_size as usize);
        let name_index = self.parse_index()?;
        let top_dict_index = self.parse_index()?;
        assert_eq!(
            top_dict_index.count, 1,
            "todo: support multiple fonts in one cff file"
        );
        let top_dict = CffDictInterpreter::parse_top_dict(top_dict_index.data)?;
        let string_index = self.parse_index()?;

        self.cursor = top_dict.char_strings.unwrap() as usize;
        let charstring_index = self.parse_index()?;

        let encoding = match top_dict.encoding {
            0 => CffEncoding::Standard,
            1 => CffEncoding::Expert,
            offset => {
                // todo: validate this - 1
                self.cursor = offset as usize - 1;
                self.parse_encoding()?
            }
        };

        let charset = match top_dict.charset {
            0 => CffCharset::IsoAdobe,
            1 => CffCharset::Expert,
            2 => CffCharset::ExpertSubset,
            offset => {
                self.cursor = offset as usize;
                self.parse_charset(charstring_index.count)?
            }
        };

        Ok(CffFile {
            name_index,
            top_dict,
            string_index,
            charstring_index,
            encoding,
            charset,
        })
    }

    fn parse_charset(&mut self, mut n_glyphs: u16) -> anyhow::Result<CffCharset> {
        Ok(match self.next()? {
            0 => {
                let name_ids = self.buffer[self.cursor..self.cursor + n_glyphs as usize * 2 - 1]
                    .chunks_exact(2)
                    .map(|b| u16::from_be_bytes([b[0], b[1]]))
                    .collect();
                CffCharset::Zero { name_ids }
            }
            1 => {
                let mut ranges = Vec::new();
                while n_glyphs - 1 > 0 {
                    let first = self.parse_u16()?;
                    let count = self.next()?;

                    n_glyphs -= count as u16 + 1;

                    ranges.push(CharsetRangeOne { first, count });
                }

                CffCharset::One(ranges)
            }
            2 => {
                let mut ranges = Vec::new();
                while n_glyphs - 1 > 0 {
                    let first = self.parse_u16()?;
                    let count = self.parse_u16()?;

                    n_glyphs -= count + 1;

                    ranges.push(CharsetRangeTwo { first, count });
                }

                CffCharset::Two(ranges)
            }
            format => anyhow::bail!("invalid charset format: {}", format),
        })
    }

    fn parse_encoding(&mut self) -> anyhow::Result<CffEncoding<'a>> {
        Ok(match self.next()? {
            0 => {
                let n_codes = self.next()?;
                let codes = &self.buffer[self.cursor..self.cursor + n_codes as usize];

                CffEncoding::Zero { n_codes, codes }
            }
            1 => {
                let n_ranges = self.next()?;
                let mut ranges = Vec::with_capacity(n_ranges as usize);
                for _ in 0..n_ranges {
                    let first = self.next()?;
                    let count = self.next()?;

                    ranges.push(EncodingRangeOne { first, count });
                }

                CffEncoding::One(ranges)
            }
            format => anyhow::bail!("invalid encoding format: {}", format),
        })
    }

    fn parse_header(&mut self) -> anyhow::Result<CffHeader> {
        let major = self.next()?;
        let minor = self.next()?;
        let header_size = self.next()?;
        let off_size = self.next()?;

        Ok(CffHeader {
            major,
            minor,
            header_size,
            off_size,
        })
    }

    fn parse_offset(&mut self, offsize: u8) -> anyhow::Result<u32> {
        Ok(match offsize {
            1 => self.next()? as u32,
            2 => self.parse_u16()? as u32,
            4 => self.parse_u32()?,
            _ => anyhow::bail!("invalid offsize: {}", offsize),
        })
    }

    fn parse_index(&mut self) -> anyhow::Result<CffIndex<'a>> {
        let count = self.parse_u16()?;
        let offsize = self.next()?;
        let mut offset = Vec::new();

        for _ in 0..=count {
            offset.push(self.parse_offset(offsize)?);
        }

        let start = self.cursor;
        let end = self.cursor + offset.last().copied().unwrap_or(0) as usize - 1;

        self.cursor = end;

        let data = &self.buffer[start..end];

        Ok(CffIndex {
            count,
            offsize,
            offset,
            data,
        })
    }
}

impl<'a> BinaryParser for CffParser<'a> {
    fn buffer(&self) -> &[u8] {
        self.buffer
    }
    fn cursor(&self) -> usize {
        self.cursor
    }
    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }
}
