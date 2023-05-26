use crate::font::true_type::{FWord, Fixed, LongDateTime};

#[derive(Debug)]
pub struct Head {
    pub font_revision: Fixed,
    pub flags: HeadFlags,
    pub units_per_em: u16,
    pub created: LongDateTime,
    pub modified: LongDateTime,
    pub x_min: FWord,
    pub y_min: FWord,
    pub x_max: FWord,
    pub y_max: FWord,
    pub mac_style: MacStyle,
    /// Smallest readable size in pixels
    pub lowest_rec_ppem: u16,
    pub font_direction_hint: i16,
    pub index_to_loc_format: i16,
    pub glyph_data_format: i16,
}

#[derive(Debug)]
pub struct HeadFlags(pub u16);

#[derive(Debug)]
pub struct MacStyle(pub u16);

impl MacStyle {
    const BOLD: u16 = 1 << 0;
    const ITALIC: u16 = 1 << 1;
    const UNDERLINE: u16 = 1 << 2;
    const OUTLINE: u16 = 1 << 3;
    const SHADOW: u16 = 1 << 4;
    const CONDENSED_NARROW: u16 = 1 << 5;
    const EXTENDED: u16 = 1 << 6;
}

// todo:
struct FontDirectionHint(i16);
