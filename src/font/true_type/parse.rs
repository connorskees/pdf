use std::fmt;

use crate::font::true_type::{table::Cmap8Group, Fixed};

use super::{
    table::{
        CmapSubtable, CmapTable, CompoundGlyphPartDescription, CvtTable, DirectoryTableEntry,
        FontDirectory, GlyfTable, Head, HeadFlags, LocaTable, MacStyle, MaxpTable, NameRecord,
        NameTable, OffsetSubtable, OutlineFlag, SimpleGlyph, TableDirectory, TableTag,
        TrueTypeGlyph,
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

    pub fn parse_simple_glyph_flags(&mut self, number_of_points: usize) -> Vec<u8> {
        let mut flags = Vec::new();

        while flags.len() < number_of_points {
            let next = self.next().unwrap();
            let should_repeat = next & 0b1000 != 0;
            flags.push(next);

            if should_repeat {
                let num_repeat = self.next().unwrap();
                for _ in 0..num_repeat {
                    flags.push(next);
                }
            }
        }

        assert_eq!(flags.len(), number_of_points);

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
        self.cursor += instruction_length as usize;

        let number_of_points = *end_points_of_contours.last().unwrap() as usize + 1;

        let flags = self.parse_simple_glyph_flags(number_of_points);

        let mut x_coords = Vec::with_capacity(number_of_points);
        let mut y_coords = Vec::with_capacity(number_of_points);

        let mut last_x = 0;
        for &flag in &flags {
            let is_short = flag & OutlineFlag::X_SHORT_VECTOR != 0;
            let is_same_or_positive = flag & OutlineFlag::X_SAME_OR_POSITIVE != 0;

            let delta_x = match (is_short, is_same_or_positive) {
                (false, false) => self.read_i16().unwrap(),
                (false, true) => {
                    x_coords.push(last_x);
                    continue;
                }
                (true, false) => -(self.next().unwrap() as i16),
                (true, true) => self.next().unwrap() as i16,
            };

            last_x += delta_x;
            x_coords.push(last_x);
        }

        let mut last_y = 0;
        for &flag in &flags {
            let is_short = flag & OutlineFlag::Y_SHORT_VECTOR != 0;
            let is_same_or_positive = flag & OutlineFlag::Y_SAME_OR_POSITIVE != 0;

            let delta_y = match (is_short, is_same_or_positive) {
                (false, false) => self.read_i16().unwrap(),
                (false, true) => {
                    y_coords.push(last_y);
                    continue;
                }
                (true, false) => -(self.next().unwrap() as i16),
                (true, true) => self.next().unwrap() as i16,
            };

            last_y += delta_y;
            y_coords.push(last_y);
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

    pub fn read_cmap_table(&mut self, table_offset: usize) -> anyhow::Result<CmapTable> {
        self.cursor = table_offset;

        let version = self.read_u16()?;
        assert_eq!(version, 0);

        let number_subtables = self.read_u16()?;
        let mut offsets = Vec::with_capacity(usize::from(number_subtables));

        for _ in 0..number_subtables {
            let platform_id = self.read_u16()?;
            let platform_specific_id = self.read_u16()?;
            let offset = self.read_u32()?;

            assert!(
                matches!(platform_id, 0 | 1 | 3),
                "invalid platform id: {:?}",
                platform_id
            );
            match platform_id {
                0 => assert!(matches!(platform_specific_id, 0..=6)),
                1 => assert!(matches!(platform_specific_id, 0..=150)),
                3 => assert!(matches!(platform_specific_id, 0..=5 | 10)),
                _ => unreachable!(),
            }

            offsets.push(offset as usize);
        }

        let mut subtables = Vec::with_capacity(usize::from(number_subtables));

        for subtable_offset in offsets {
            self.cursor = table_offset + subtable_offset;
            subtables.push(self.parse_cmap_subtable()?);
        }

        Ok(CmapTable { version, subtables })
    }

    fn parse_cmap_subtable(&mut self) -> anyhow::Result<CmapSubtable> {
        let format = self.read_u16()?;

        match format {
            0 => self.parse_cmap_subtable_0(),
            2 => todo!(),
            4 => self.parse_cmap_subtable_4(),
            6 => self.parse_cmap_subtable_6(),
            8 => self.parse_cmap_subtable_8(),
            10 => todo!(),
            12 => todo!(),
            13 => todo!(),
            14 => todo!(),
            _ => anyhow::bail!("invalid cmap subtable format: {:?}", format),
        }
    }

    fn parse_cmap_subtable_0(&mut self) -> anyhow::Result<CmapSubtable> {
        let length = self.read_u16()?;
        assert_eq!(length, 262, "length must be 262 for type 0 cmap subtable");
        let language = self.read_u16()?;
        let glyph_index_array = self.get_byte_range(256).try_into()?;

        Ok(CmapSubtable::Zero {
            language,
            glyph_index_array,
        })
    }

    fn parse_cmap_subtable_4(&mut self) -> anyhow::Result<CmapSubtable> {
        let start_pos = self.cursor - 2;
        let length = self.read_u16()?;
        let language = self.read_u16()?;
        let seg_count_x2 = self.read_u16()?;
        let search_range = self.read_u16()?;
        let entry_selector = self.read_u16()?;
        let range_shift = self.read_u16()?;

        let seg_count = seg_count_x2 / 2;

        let mut end_code = Vec::with_capacity(seg_count as usize);
        for _ in 0..seg_count {
            end_code.push(self.read_u16()?);
        }

        assert_eq!(end_code.last(), Some(&0xFFFF));

        let reserved_pad = self.read_u16()?;
        assert_eq!(reserved_pad, 0);

        let mut start_code = Vec::with_capacity(seg_count as usize);
        for _ in 0..seg_count {
            start_code.push(self.read_u16()?);
        }

        let mut id_delta = Vec::with_capacity(seg_count as usize);
        for _ in 0..seg_count {
            id_delta.push(self.read_i16()?);
        }

        let mut id_range_offset = Vec::with_capacity(seg_count as usize);
        for _ in 0..seg_count {
            id_range_offset.push(self.read_u16()?);
        }

        assert!(start_pos + length as usize >= self.cursor);

        let glyph_index_array = self
            .get_byte_range(start_pos + length as usize - self.cursor)
            .chunks_exact(2)
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
            .collect();

        Ok(CmapSubtable::Four {
            language,
            seg_count_x2,
            search_range,
            entry_selector,
            range_shift,
            end_code,
            start_code,
            id_delta,
            id_range_offset,
            glyph_index_array,
        })
    }

    fn parse_cmap_subtable_6(&mut self) -> anyhow::Result<CmapSubtable> {
        let start = self.cursor - 2;
        let length = self.read_u16()?;
        let language = self.read_u16()?;
        let first_code = self.read_u16()?;
        let entry_count = self.read_u16()?;

        let mut glyph_index_array = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            glyph_index_array.push(self.read_u16()?);
        }

        assert_eq!(start + length as usize, self.cursor);

        Ok(CmapSubtable::Six {
            language,
            first_code,
            entry_count,
            glyph_index_array,
        })
    }

    fn parse_cmap_subtable_8(&mut self) -> anyhow::Result<CmapSubtable> {
        let start = self.cursor - 2;
        let reserved = self.read_u16()?;
        assert_eq!(reserved, 0);
        let length = self.read_u32()?;
        let language = self.read_u32()?;
        let is32 = self.get_byte_range(65536).to_vec();
        let n_groups = self.read_u32()?;

        let mut groups = Vec::with_capacity(n_groups as usize);

        for _ in 0..n_groups {
            let start_char_code = self.read_u32()?;
            let end_char_code = self.read_u32()?;
            let start_glyph_code = self.read_u32()?;

            groups.push(Cmap8Group {
                start_char_code,
                end_char_code,
                start_glyph_code,
            })
        }

        assert_eq!(self.cursor, start + length as usize);

        Ok(CmapSubtable::Eight {
            language,
            is32,
            groups,
        })
    }
}
