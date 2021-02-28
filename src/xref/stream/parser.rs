use std::{collections::HashMap, convert::TryFrom, mem};

use crate::{xref::Xref, PdfResult};

use crate::xref::{
    stream::{XrefStreamField, XrefStreamFieldWidths},
    XrefEntry,
};

#[derive(Debug)]
pub(crate) struct XrefStreamParser<'a> {
    stream: &'a [u8],
    cursor: usize,
    w: XrefStreamFieldWidths,
    index: Vec<(usize, usize)>,
}

fn strip_leading_zeroes(bytes: &[u8]) -> &[u8] {
    let mut start = 0;

    while bytes.get(start) == Some(&0) {
        start += 1;
    }

    &bytes[start..]
}

fn parse_integer(bytes: &[u8], default: Option<u64>) -> PdfResult<u64> {
    if bytes.is_empty() {
        return match default {
            Some(v) => Ok(v),
            None => todo!(),
        };
    }

    let bytes = strip_leading_zeroes(bytes);

    if bytes.len() > 8 {
        todo!()
    }

    let mut sum = 0;

    for &b in bytes {
        sum <<= 8;
        sum += b as u64;
    }

    Ok(sum)
}

impl<'a> XrefStreamParser<'a> {
    pub fn new(stream: &'a [u8], w: XrefStreamFieldWidths, index: Vec<(usize, usize)>) -> Self {
        Self {
            w,
            index,
            stream,
            cursor: 0,
        }
    }

    pub fn parse(mut self) -> PdfResult<Xref> {
        let mut objects = HashMap::new();

        for (idx_offset, num_of_objects) in mem::take(&mut self.index) {
            for idx in 0..num_of_objects {
                objects.insert(idx + idx_offset, self.parse_entry()?);
            }
        }

        debug_assert_eq!(self.cursor, self.stream.len());

        Ok(Xref { objects })
    }

    fn parse_entry(&mut self) -> PdfResult<XrefEntry> {
        let entry_type = parse_integer(self.next_field(XrefStreamField::One), Some(1))?;

        match entry_type {
            0 => self.parse_type_zero_entry(),
            1 => self.parse_type_one_entry(),
            2 => self.parse_type_two_entry(),
            _ => self.parse_type_unknown_entry(),
        }
    }

    /// Equivalent to free entries in a regular xref table
    fn parse_type_zero_entry(&mut self) -> PdfResult<XrefEntry> {
        let next_free_object = parse_integer(self.next_field(XrefStreamField::Two), None)?;
        let generation_number = u16::try_from(parse_integer(
            self.next_field(XrefStreamField::Three),
            None,
        )?)?;

        Ok(XrefEntry::Free {
            next_free_object,
            generation_number,
        })
    }

    /// Equivalent to in-use entires in a regular xref table
    fn parse_type_one_entry(&mut self) -> PdfResult<XrefEntry> {
        let byte_offset =
            usize::try_from(parse_integer(self.next_field(XrefStreamField::Two), None)?)?;
        let generation_number = u16::try_from(parse_integer(
            self.next_field(XrefStreamField::Three),
            Some(0),
        )?)?;

        Ok(XrefEntry::InUse {
            byte_offset,
            generation_number,
        })
    }

    /// Compressed xref entries
    fn parse_type_two_entry(&mut self) -> PdfResult<XrefEntry> {
        let object_number = parse_integer(self.next_field(XrefStreamField::Two), None)?;
        let index = usize::try_from(parse_integer(
            self.next_field(XrefStreamField::Three),
            None,
        )?)?;

        Ok(XrefEntry::Compressed {
            object_number,
            index,
        })
    }

    fn parse_type_unknown_entry(&mut self) -> PdfResult<XrefEntry> {
        self.next_field(XrefStreamField::Two);
        self.next_field(XrefStreamField::Three);

        Ok(XrefEntry::Null)
    }

    fn next_field(&mut self, field: XrefStreamField) -> &'a [u8] {
        let field_width = self.w.field_width(field);
        // todo: this will panic
        let field_val = &self.stream[self.cursor..(self.cursor + field_width)];
        self.cursor += field_width;

        field_val
    }
}
