#[derive(Debug)]
pub struct CmapTable {
    /// Version number (Set to zero)
    pub version: u16,
    pub subtables: Vec<CmapSubtable>,
}

#[derive(Debug)]
pub enum CmapSubtable {
    /// Format 0 is suitable for fonts whose character codes and glyph indices are
    /// restricted to single bytes. This was a very common situation when
    /// TrueType was introduced but is rarely encountered now
    Zero {
        language: u16,
        glyph_index_array: [u8; 256],
    },
    Two {
        language: u16,
        sub_header_keys: [u16; 256],
        subheaders: Vec<Cmap2SubHeader>,
        glyph_index_array: Vec<u16>,
    },

    /// Format 4 is a two-byte encoding format. It should be used when the character
    /// codes for a font fall into several contiguous ranges, possibly with holes
    /// in some or all of the ranges. That is, some of the codes in a range may
    /// not be associated with glyphs in the font. Two-byte fonts that are
    /// densely mapped should use Format 6
    Four {
        language: u16,
        seg_count_x2: u16,
        search_range: u16,
        entry_selector: u16,
        range_shift: u16,
        end_code: Vec<u16>,
        start_code: Vec<u16>,
        id_delta: Vec<i16>,
        id_range_offset: Vec<u16>,
        glyph_index_array: Vec<u16>,
    },

    /// Format 6 is used to map 16-bit, 2-byte, characters to glyph indexes. It is
    /// sometimes called the trimmed table mapping. It should be used when
    /// character codes for a font fall into a single contiguous range. This
    /// results in what is termed a dense mapping. Two-byte fonts that are not
    /// densely mapped (due to their multiple contiguous ranges) should use
    /// Format 4
    Six {
        language: u16,
        first_code: u16,
        entry_count: u16,
        glyph_index_array: Vec<u16>,
    },

    /// Mixed 16-bit and 32-bit coverage
    ///
    /// Format 8 is a bit like format 2, in that it provides for mixed-length
    /// character codes. If a font contains Unicode surrogates, it's likely that
    /// it will also include other, regular 16-bit Unicodes as well. This
    /// requires a format to map a mixture of 16-bit and 32-bit character codes,
    /// just as format 2 allows a mixture of 8-bit and 16-bit codes. A
    /// simplifying assumption is made: namely, that there are no 32-bit
    /// character codes which share the same first 16 bits as any 16-bit
    /// character code. This means that the determination as to whether a
    /// particular 16-bit value is a standalone character code or the start of a
    /// 32-bit character code can be made by looking at the 16-bit value
    /// directly, with no further information required
    Eight {
        language: u32,
        is32: Vec<u8>,
        groups: Vec<Cmap8Group>,
    },
    Ten {
        language: u32,
        start_char_code: u32,
        num_chars: u32,
        glyphs: Vec<u16>,
    },
    Twelve,
    Thirteen,
    Fourteen,
}

impl CmapSubtable {
    pub fn lookup_char_code(&self, char_code: u32) -> u32 {
        match self {
            CmapSubtable::Four {
                end_code,
                start_code,
                id_delta,
                id_range_offset,
                seg_count_x2,
                glyph_index_array,
                ..
            } => {
                let (idx, _) = end_code
                    .iter()
                    .enumerate()
                    .find(|(_, end_code)| **end_code as u32 >= char_code)
                    .unwrap();

                let start_code = start_code[idx];
                if start_code as u32 > char_code {
                    return 0;
                }

                if id_range_offset[idx] == 0 {
                    (id_delta[idx] as i32 + char_code as i32) as u32 % 65536
                } else {
                    let addr = glyph_index_array[(idx as u16
                        + id_range_offset[idx] / 2
                        + (char_code as u16 - start_code)
                        - *seg_count_x2 / 2)
                        as usize];

                    (id_delta[idx] as i32 + addr as i32) as u32 % 65536
                }
            }
            CmapSubtable::Zero {
                glyph_index_array, ..
            } => {
                assert!(char_code <= 256);
                glyph_index_array[char_code as usize] as u32
            }
            CmapSubtable::Six {
                glyph_index_array,
                first_code,
                ..
            } => {
                println!("likely incorrect ttf cmap subtable 6");
                if char_code == 0 {
                    return *first_code as u32;
                }

                glyph_index_array
                    .get(char_code as usize - 1)
                    .copied()
                    .unwrap_or(0) as u32
            }
            _ => todo!("unimplemented cmap table lookup: {:#?}", self),
        }
    }
}

#[derive(Debug)]
pub struct Cmap8Group {
    /// First character code in this group; note that if this group is for one or
    /// more 16-bit character codes (which is determined from the is32 array),
    /// this 32-bit value will have the high 16-bits set to zero
    pub start_char_code: u32,

    /// Last character code in this group; same condition as listed above for the startCharCode
    pub end_char_code: u32,

    /// Glyph index corresponding to the starting character code
    pub start_glyph_code: u32,
}

#[derive(Debug)]
pub struct Cmap2SubHeader {
    first_code: u16,
    entry_count: u16,
    id_delta: i16,
    id_range_offset: u16,
}
