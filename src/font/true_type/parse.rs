use std::fmt;

use crate::font::true_type::Fixed;

use super::{
    table::{
        CompoundGlyphPartDescription, CvtTable, DirectoryTableEntry, FontDirectory, GlyfTable,
        Head, HeadFlags, LocaTable, MacStyle, MaxpTable, NameRecord, NameTable, OffsetSubtable,
        SimpleGlyph, TableDirectory, TableTag, TrueTypeGlyph,
    },
    FWord, LongDateTime,
};

pub(super) struct TrueTypeParser<'a> {
    pub buffer: &'a [u8],
    pub cursor: usize,
}

impl fmt::Debug for TrueTypeParser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrueTypeParser")
            .field("cursor", &self.cursor)
            .field("buffer", &format!("[ {} bytes ]", self.buffer.len()))
            .finish()
    }
}

/// Base parsing
impl<'a> TrueTypeParser<'a> {
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

    fn read_u16(&mut self) -> anyhow::Result<u16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(u16::from_be_bytes([b1, b2]))
    }

    fn read_i16(&mut self) -> anyhow::Result<i16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Ok(i16::from_be_bytes([b1, b2]))
    }

    fn read_u32_bytes(&mut self) -> anyhow::Result<[u8; 4]> {
        Ok(self.read_u32()?.to_be_bytes())
    }

    fn read_u32(&mut self) -> anyhow::Result<u32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(u32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn read_u64(&mut self) -> anyhow::Result<u64> {
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

    fn read_i64(&mut self) -> anyhow::Result<i64> {
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

    fn read_fixed(&mut self) -> anyhow::Result<Fixed> {
        let n = self.read_u32()?;

        Ok(Fixed(i32::from_be_bytes(n.to_be_bytes())))
    }

    fn read_fword(&mut self) -> anyhow::Result<FWord> {
        Ok(FWord(self.read_i16()?))
    }

    fn read_long_date_time(&mut self) -> anyhow::Result<LongDateTime> {
        Ok(LongDateTime(self.read_i64()?))
    }

    #[track_caller]
    pub fn get_byte_range(&self, length: usize) -> &[u8] {
        &self.buffer[self.cursor..(self.cursor + length)]
    }
}

/// Table parsing
impl<'a> TrueTypeParser<'a> {
    fn read_offset_subtable(&mut self) -> anyhow::Result<OffsetSubtable> {
        let _sfnt_version = self.read_u32()?;
        let number_of_tables = self.read_u16()?;
        let search_range = self.read_u16()?;
        let entry_selector = self.read_u16()?;
        let range_shift = self.read_u16()?;

        Ok(OffsetSubtable {
            number_of_tables,
            search_range,
            entry_selector,
            range_shift,
        })
    }

    fn read_tag(&mut self) -> anyhow::Result<TableTag> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Ok(TableTag::new([b1, b2, b3, b4]))
    }

    fn read_dir_table_entry(&mut self) -> anyhow::Result<DirectoryTableEntry> {
        let tag = self.read_tag()?;
        let checksum = self.read_u32()?;
        let offset = self.read_u32()?;
        let length = self.read_u32()?;

        Ok(DirectoryTableEntry {
            tag,
            checksum,
            offset,
            length,
        })
    }

    pub fn read_font_directory(&mut self) -> anyhow::Result<FontDirectory> {
        let offset_subtable = self.read_offset_subtable()?;
        let mut table_directory_entries =
            Vec::with_capacity(usize::from(offset_subtable.number_of_tables));

        for _ in 0..offset_subtable.number_of_tables {
            table_directory_entries.push(self.read_dir_table_entry()?);
        }

        Ok(FontDirectory {
            offset_subtable,
            table_directory: TableDirectory(table_directory_entries),
        })
    }

    pub fn parse_simple_glyph_flags(&mut self, number_of_contours: i16) -> Vec<u8> {
        let mut flags = Vec::new();

        while flags.len() < number_of_contours as usize {
            let next = self.next().unwrap();
            let should_repeat = next & 0b1000 != 0;
            flags.push(next);
            if should_repeat {
                let num_repeat = self.next().unwrap();
                for _ in 0..num_repeat {
                    // flags.push(next);
                }
            }
        }

        assert_eq!(flags.len(), number_of_contours as usize);

        flags
    }

    fn parse_simple_glyph(&mut self, number_of_contours: i16) -> anyhow::Result<SimpleGlyph> {
        let mut end_points_of_contours = Vec::with_capacity(number_of_contours as usize);

        // todo: this should just reinterpret bytes
        for _ in 0..number_of_contours {
            end_points_of_contours.push(self.read_u16().unwrap());
        }

        let instruction_length = self.read_u16().unwrap();
        let instructions = self.get_byte_range(instruction_length as usize).to_vec();

        let flags = self.parse_simple_glyph_flags(number_of_contours);

        let mut x_coords = Vec::new();
        let mut y_coords = Vec::new();

        for &flag in &flags {
            if flag & 0b10 != 0 {
                x_coords.push(self.next().unwrap() as u16);
            } else {
                x_coords.push(self.read_u16().unwrap());
            }
        }

        for &flag in &flags {
            if flag & 0b100 != 0 {
                y_coords.push(self.next().unwrap() as u16);
            } else {
                y_coords.push(self.read_u16().unwrap());
            }
        }

        Ok(SimpleGlyph {
            end_points_of_contours,
            instructions,
            flags,
            x_coords,
            y_coords,
        })
    }

    fn parse_compound_glyph(&mut self) -> anyhow::Result<Vec<CompoundGlyphPartDescription>> {
        todo!()
    }

    pub fn parse_glyph(&mut self) -> anyhow::Result<TrueTypeGlyph> {
        let number_of_contours = self.read_i16().unwrap();
        let _x_min = self.read_fword().unwrap();
        let _y_min = self.read_fword().unwrap();
        let _x_max = self.read_fword().unwrap();
        let _y_max = self.read_fword().unwrap();

        Ok(
            if number_of_contours.is_positive() || number_of_contours == 0 {
                let glyph = self.parse_simple_glyph(number_of_contours).unwrap();

                TrueTypeGlyph::Simple(glyph)
            } else {
                let glyph = self.parse_compound_glyph().unwrap();

                TrueTypeGlyph::Compound(glyph)
            },
        )
    }

    pub fn read_glyf_table(
        &mut self,
        offset: usize,
        num_glyphs: usize,
    ) -> anyhow::Result<GlyfTable> {
        self.cursor = offset;

        let mut glyphs = Vec::with_capacity(num_glyphs);

        for _ in 0..num_glyphs {
            glyphs.push(self.parse_glyph()?);
        }

        Ok(GlyfTable { glyphs })
    }

    pub fn read_loca_table(
        &mut self,
        offset: usize,
        length: usize,
        format: i16,
    ) -> anyhow::Result<LocaTable> {
        self.cursor = offset;

        let buffer = self.get_byte_range(length);

        let offsets = match format {
            // short
            0 => buffer
                .chunks_exact(2)
                .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]) as u32 * 2)
                .collect(),
            // long
            1 => buffer
                .chunks_exact(4)
                .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .collect(),
            _ => todo!("unsupported loca table format: {:?}", format),
        };

        Ok(LocaTable { offsets })
    }

    pub fn read_head_table(&mut self, offset: usize) -> anyhow::Result<Head> {
        self.cursor = offset;

        let version = self.read_u32()?;
        assert_eq!(version, 0x00010000);
        let font_revision = self.read_fixed()?;
        // todo: perhaps verify this?
        let _check_sum_adjustment = self.read_u32()?;
        let magic_number = self.read_u32()?;
        assert_eq!(magic_number, 0x5F0F3CF5);

        let flags = HeadFlags(self.read_u16()?);
        let units_per_em = self.read_u16()?;
        let created = self.read_long_date_time()?;
        let modified = self.read_long_date_time()?;
        let x_min = self.read_fword()?;
        let y_min = self.read_fword()?;
        let x_max = self.read_fword()?;
        let y_max = self.read_fword()?;
        let mac_style = MacStyle(self.read_u16()?);
        let lowest_rec_ppem = self.read_u16()?;
        let font_direction_hint = self.read_i16()?;
        let index_to_loc_format = self.read_i16()?;
        let glyph_data_format = self.read_i16()?;

        Ok(Head {
            font_revision,
            flags,
            units_per_em,
            created,
            modified,
            x_min,
            y_min,
            x_max,
            y_max,
            mac_style,
            lowest_rec_ppem,
            font_direction_hint,
            index_to_loc_format,
            glyph_data_format,
        })
    }

    pub fn read_maxp_table(&mut self, offset: usize) -> anyhow::Result<MaxpTable> {
        self.cursor = offset;

        let version = self.read_u32()?;
        assert_eq!(version, 0x00010000);

        let num_glyphs = self.read_u16()?;
        let max_points = self.read_u16()?;
        let max_contours = self.read_u16()?;
        let max_component_points = self.read_u16()?;
        let max_component_contours = self.read_u16()?;
        let max_zones = self.read_u16()?;
        let max_twilight_points = self.read_u16()?;
        let max_storage = self.read_u16()?;
        let max_function_defs = self.read_u16()?;
        let max_instruction_defs = self.read_u16()?;
        let max_stack_elements = self.read_u16()?;
        let max_size_of_instructions = self.read_u16()?;
        let max_component_elements = self.read_u16()?;
        let max_component_depth = self.read_u16()?;

        Ok(MaxpTable {
            version: Fixed(i32::from_be_bytes(version.to_be_bytes())),
            num_glyphs,
            max_points,
            max_contours,
            max_component_points,
            max_component_contours,
            max_zones,
            max_twilight_points,
            max_storage,
            max_function_defs,
            max_instruction_defs,
            max_stack_elements,
            max_size_of_instructions,
            max_component_elements,
            max_component_depth,
        })
    }

    pub fn read_name_table(&mut self, offset: usize) -> anyhow::Result<NameTable> {
        self.cursor = offset;
        let format = self.read_u16()?;
        let count = self.read_u16()?;
        let string_offset = self.read_u16()?;

        let mut name_records = Vec::with_capacity(count as usize);

        for _ in 0..count {
            name_records.push(self.read_name_record(offset + string_offset as usize)?);
        }

        Ok(NameTable {
            format,
            string_offset,
            name_records,
        })
    }

    fn read_name_record(&mut self, string_offset: usize) -> anyhow::Result<NameRecord> {
        let platform_id = self.read_u16()?;
        let platform_specific_id = self.read_u16()?;
        let language_id = self.read_u16()?;
        let name_id = self.read_u16()?;
        let length = self.read_u16()?;
        let offset = self.read_u16()?;

        Ok(NameRecord {
            platform_id,
            platform_specific_id,
            language_id,
            name_id,
            length,
            offset,
            absolute_offset: offset as usize + string_offset,
        })
    }

    pub fn read_cvt_table(&mut self, entry: DirectoryTableEntry) -> anyhow::Result<CvtTable> {
        self.cursor = entry.offset as usize;

        let num_entries = entry.length as usize / 4;

        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(self.read_fword()?);
        }

        Ok(CvtTable { entries })
    }
}
