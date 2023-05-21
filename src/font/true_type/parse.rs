use std::fmt;

use super::table::{
    CompoundGlyphPartDescription, DirectoryTableEntry, FontDirectory, GlyfTable, Glyph, Head,
    OffsetSubtable, SimpleGlyph, TableDirectory, TableTag,
};

#[derive(Debug)]
pub(crate) struct TrueTypeFontFile<'a> {
    font_directory: FontDirectory,
    parser: TrueTypeParser<'a>,
}

impl<'a> TrueTypeFontFile<'a> {
    pub fn new(buffer: &'a [u8]) -> Option<Self> {
        let mut parser = TrueTypeParser::new(buffer);

        let font_directory = parser.read_font_directory()?;

        Some(Self {
            parser,
            font_directory,
        })
    }

    pub fn glyphs(&mut self) -> () {
        let offset = self
            .font_directory
            .find_table_offset(GlyfTable::TAG)
            .unwrap();
        self.parser.read_glyf_table(offset as usize);
    }
}

struct TrueTypeParser<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

impl fmt::Debug for TrueTypeParser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrueTypeParser")
            .field("cursor", &self.cursor)
            .field("buffer", &format!("[ {} bytes ]", self.buffer.len()))
            .finish()
    }
}

impl<'a> TrueTypeParser<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn next(&mut self) -> Option<u8> {
        self.buffer.get(self.cursor).map(|b| {
            self.cursor += 1;
            *b
        })
    }

    fn read_u16(&mut self) -> Option<u16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Some(u16::from_be_bytes([b1, b2]))
    }

    fn read_i16(&mut self) -> Option<i16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Some(i16::from_be_bytes([b1, b2]))
    }

    fn read_u32_bytes(&mut self) -> Option<[u8; 4]> {
        Some(self.read_u32()?.to_be_bytes())
    }

    fn read_u32(&mut self) -> Option<u32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Some(u32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn read_fword(&mut self) -> Option<i16> {
        todo!()
    }

    fn read_offset_subtable(&mut self) -> Option<OffsetSubtable> {
        let _sfnt_version = self.read_u32()?;
        let number_of_tables = self.read_u16()?;
        let search_range = self.read_u16()?;
        let entry_selector = self.read_u16()?;
        let range_shift = self.read_u16()?;

        Some(OffsetSubtable {
            number_of_tables,
            search_range,
            entry_selector,
            range_shift,
        })
    }

    fn read_tag(&mut self) -> Option<TableTag> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Some(TableTag::new([b1, b2, b3, b4]))
    }

    fn read_dir_table_entry(&mut self) -> Option<DirectoryTableEntry> {
        let tag = self.read_tag()?;
        let checksum = self.read_u32()?;
        let offset = self.read_u32()?;
        let length = self.read_u32()?;

        Some(DirectoryTableEntry {
            tag,
            checksum,
            offset,
            length,
        })
    }

    fn read_font_directory(&mut self) -> Option<FontDirectory> {
        let offset_subtable = self.read_offset_subtable()?;
        let mut table_directory_entries =
            Vec::with_capacity(usize::from(offset_subtable.number_of_tables));

        for _ in 0..offset_subtable.number_of_tables {
            table_directory_entries.push(self.read_dir_table_entry()?);
        }

        Some(FontDirectory {
            offset_subtable,
            table_directory: TableDirectory(table_directory_entries),
        })
    }

    fn parse_head(&mut self) -> Option<Head> {
        todo!()
    }

    #[track_caller]
    fn get_byte_range(&self, length: usize) -> &[u8] {
        &self.buffer[self.cursor..(self.cursor + length)]
    }

    fn parse_simple_glyph_flags(&mut self) -> Vec<u8> {
        todo!()
    }

    fn parse_simple_glyph(&mut self, number_of_contours: i16) -> Option<SimpleGlyph> {
        let mut end_points_of_contours = Vec::new();

        // todo: this should just reinterpret bytes
        for _ in 0..number_of_contours {
            end_points_of_contours.push(self.read_u16()?);
        }

        let instruction_length = self.read_u16()?;
        let instructions = self.get_byte_range(instruction_length as usize).to_vec();

        let flag_start = self.cursor;

        // let flag = self.next().unwrap();
        let flags = self.parse_simple_glyph_flags();

        let mut x_coords = Vec::new();
        let mut y_coords = Vec::new();

        for &flag in &flags {
            if true {
                x_coords.push(self.next()? as u16);
            } else {
                x_coords.push(self.read_u16()?);
            }
        }

        for &flag in &flags {
            if true {
                y_coords.push(self.next()? as u16);
            } else {
                y_coords.push(self.read_u16()?);
            }
        }
        // println!("{:#b}", flag);
        // let flags = &[self.next()?];

        Some(SimpleGlyph {
            end_points_of_contours,
            instructions,
            flags,
            x_coords,
            y_coords,
        })
    }

    fn parse_compound_glyph(&mut self) -> Option<Vec<CompoundGlyphPartDescription>> {
        todo!()
    }

    fn parse_glyph(&mut self) -> Option<Glyph> {
        let number_of_contours = self.read_i16()?;
        let x_min = self.read_u16()?;
        let y_min = self.read_u16()?;
        let x_max = self.read_u16()?;
        let y_max = self.read_u16()?;

        dbg!(number_of_contours);

        Some(
            if number_of_contours.is_positive() || number_of_contours == 0 {
                let glyph = self.parse_simple_glyph(number_of_contours)?;

                Glyph::Simple(glyph)
            } else {
                let glyph = self.parse_compound_glyph()?;

                Glyph::Compound(glyph)
            },
        )
    }

    fn read_glyf_table(&mut self, offset: usize) -> Option<GlyfTable> {
        self.cursor = offset;

        let mut glyphs = Vec::new();

        loop {
            glyphs.push(self.parse_glyph()?);
        }

        Some(GlyfTable { glyphs })
    }
}
