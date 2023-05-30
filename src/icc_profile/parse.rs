use crate::{
    date::Date,
    error::PdfResult,
    icc_profile::{IccProfileHeader, IccTagSignature},
};

use super::{
    data_types::{F15Dot16, XyzNumber},
    IccProfile, IccTagTable, TagTableEntry,
};

pub struct IccProfileParser<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

// todo: methods in this block should be part of shared trait between all binary
// parsers
impl<'a> IccProfileParser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn next(&mut self) -> anyhow::Result<u8> {
        self.buffer
            .get(self.cursor)
            .map(|b| {
                self.cursor += 1;
                *b
            })
            .ok_or(anyhow::anyhow!("unexpected eof"))
    }

    fn parse_u16(&mut self) -> anyhow::Result<u16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(u16::from_be_bytes([b1, b2]))
    }

    fn parse_i16(&mut self) -> anyhow::Result<i16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(i16::from_be_bytes([b1, b2]))
    }

    fn parse_u32_bytes(&mut self) -> anyhow::Result<[u8; 4]> {
        Ok(self.parse_u32()?.to_be_bytes())
    }

    fn parse_u32(&mut self) -> anyhow::Result<u32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(u32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn parse_i32(&mut self) -> anyhow::Result<i32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(i32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn parse_u64(&mut self) -> anyhow::Result<u64> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;
        let b5 = self.next()?;
        let b6 = self.next()?;
        let b7 = self.next()?;
        let b8 = self.next()?;

        Ok(u64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
    }

    fn parse_i64(&mut self) -> anyhow::Result<i64> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;
        let b5 = self.next()?;
        let b6 = self.next()?;
        let b7 = self.next()?;
        let b8 = self.next()?;

        Ok(i64::from_be_bytes([b1, b2, b3, b4, b5, b6, b7, b8]))
    }

    fn get_byte_range(&mut self, len: usize) -> anyhow::Result<&[u8]> {
        let buffer = &self.buffer[self.cursor..(self.cursor + len)];
        self.cursor += len;
        Ok(buffer)
    }

    fn parse_array<const LEN: usize>(&mut self) -> anyhow::Result<[u8; LEN]> {
        let slice = self.get_byte_range(LEN)?;

        Ok(<[u8; LEN]>::try_from(slice)?)
    }
}

impl<'a> IccProfileParser<'a> {
    pub fn parse(&mut self) -> PdfResult<IccProfile> {
        let header = self.parse_header()?;
        let tag_table: IccTagTable = self.parse_tag_table()?;

        Ok(IccProfile { header, tag_table })
    }

    fn parse_f15dot16(&mut self) -> PdfResult<F15Dot16> {
        Ok(F15Dot16::from_bits(self.parse_i32()?))
    }

    fn parse_date(&mut self) -> PdfResult<Date> {
        let year = self.parse_u16()?;
        let month = self.parse_u16()?;
        let day = self.parse_u16()?;

        let hour = self.parse_u16()?;
        let minute = self.parse_u16()?;
        let second = self.parse_u16()?;

        Ok(Date {
            year: Some(year),
            month: Some(month),
            day: Some(day),
            hour: Some(hour),
            minute: Some(minute),
            second: Some(second),
            ut_relationship: None,
            ut_hour_offset: None,
            ut_minute_offset: None,
        })
    }

    fn parse_xyz_number(&mut self) -> PdfResult<XyzNumber> {
        let cie_x: F15Dot16 = self.parse_f15dot16()?;
        let cie_y: F15Dot16 = self.parse_f15dot16()?;
        let cie_z: F15Dot16 = self.parse_f15dot16()?;

        Ok(XyzNumber {
            cie_x,
            cie_y,
            cie_z,
        })
    }

    fn parse_header(&mut self) -> PdfResult<IccProfileHeader> {
        let profile_size = self.parse_u32()?;
        let preferred_cmm_type = IccTagSignature(self.parse_array::<4>()?);
        let profile_version_number = self.parse_u32()?;
        let profile_device_class = IccTagSignature(self.parse_array::<4>()?);
        let colour_space = IccTagSignature(self.parse_array::<4>()?);
        let profile_connection_space = IccTagSignature(self.parse_array()?);
        let created_at = self.parse_date()?;
        let acsp = self.parse_u32()?;
        let primary_platform_signature = IccTagSignature(self.parse_array()?);
        let profile_flags = self.parse_u32()?;
        let device_manufacturer = IccTagSignature(self.parse_array()?);
        let device_model = IccTagSignature(self.parse_array()?);
        let device_attributes = self.parse_u64()?;
        let rendering_intent = self.parse_u32()?;
        let xyz_values = self.parse_xyz_number()?;
        let profile_creator_signature = IccTagSignature(self.parse_array()?);
        let profile_id = self.parse_array::<16>()?;
        let reserved = self.parse_array::<28>()?;

        anyhow::ensure!(acsp == 0x61637370);
        anyhow::ensure!(reserved.iter().all(|b| *b == 0));

        Ok(IccProfileHeader {
            profile_size,
            preferred_cmm_type,
            profile_version_number,
            profile_device_class,
            colour_space,
            profile_connection_space,
            created_at,
            primary_platform_signature,
            profile_flags,
            device_manufacturer,
            device_model,
            device_attributes,
            rendering_intent,
            xyz_values,
            profile_creator_signature,
            profile_id,
            reserved,
        })
    }

    fn parse_tag_table(&mut self) -> PdfResult<IccTagTable> {
        let tag_count = self.parse_u32()?;

        let mut entries = Vec::with_capacity(tag_count as usize);
        for _ in 0..tag_count {
            entries.push(self.parse_tag_entry()?);
        }

        Ok(IccTagTable { tag_count, entries })
    }

    fn parse_tag_entry(&mut self) -> PdfResult<TagTableEntry> {
        let signature = self.parse_u32()?;
        let offset = self.parse_u32()?;
        let len = self.parse_u32()?;

        Ok(TagTableEntry {
            signature,
            offset,
            len,
        })
    }

    fn expect_tag(&mut self, expected: IccTagSignature) -> PdfResult<()> {
        let found = IccTagSignature(self.parse_array::<4>()?);

        anyhow::ensure!(expected == found);

        Ok(())
    }

    fn parse_text_tag(&mut self, entry: TagTableEntry) -> PdfResult<TextTag> {
        self.cursor = entry.offset as usize;
        self.expect_tag(IccTagSignature(*b"text"))?;

        let reserved = self.parse_array::<4>()?;
        let string = std::str::from_utf8(self.get_byte_range(entry.len as usize - 8)?)?;

        Ok(TextTag { reserved, string })
    }

    fn parse_xyz_tag(&mut self, entry: TagTableEntry) -> PdfResult<XyzTag> {
        self.cursor = entry.offset as usize;
        let end = self.cursor + entry.len as usize;
        self.expect_tag(IccTagSignature(*b"XYZ "))?;

        let reserved = self.parse_array::<4>()?;
        let mut values = Vec::new();
        while self.cursor < end {
            values.push(self.parse_xyz_number()?);
        }

        Ok(XyzTag { reserved, values })
    }

    fn parse_curve_tag(&mut self, entry: TagTableEntry) -> PdfResult<CurveTag> {
        self.cursor = entry.offset as usize;
        self.expect_tag(IccTagSignature(*b"curv"))?;

        let reserved = self.parse_array::<4>()?;
        let count = self.parse_u32()?;
        let mut values = Vec::new();

        for _ in 0..count {
            values.push(self.parse_u16()?);
        }

        Ok(CurveTag { reserved, values })
    }

    fn parse_signature_tag(&mut self, entry: TagTableEntry) -> PdfResult<SignatureTag> {
        self.cursor = entry.offset as usize;
        self.expect_tag(IccTagSignature(*b"sig "))?;

        let reserved = self.parse_array::<4>()?;
        let signature = self.parse_u32()?;

        Ok(SignatureTag {
            reserved,
            signature,
        })
    }
}

#[derive(Debug)]
pub(super) struct TextTag<'a> {
    reserved: [u8; 4],
    string: &'a str,
}

#[derive(Debug)]
pub(super) struct XyzTag {
    reserved: [u8; 4],
    values: Vec<XyzNumber>,
}

#[derive(Debug)]
pub(super) struct CurveTag {
    reserved: [u8; 4],
    // if len is 1, this is interpreted as a Fixed8Dot8
    values: Vec<u16>,
}

#[derive(Debug)]
pub(super) struct MeasurementTag {
    reserved: [u8; 4],
    // todo:
}

#[derive(Debug)]
pub(super) struct ViewingConditionsTag {
    reserved: [u8; 4],
    // todo:
}

#[derive(Debug)]
pub(super) struct SignatureTag {
    reserved: [u8; 4],
    signature: u32,
}
