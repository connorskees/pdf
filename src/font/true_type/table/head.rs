use crate::font::true_type::{FWord, LongDateTime};

pub struct Head {
    flags: HeadFlags,
    units_per_em: u16,
    created: LongDateTime,
    modified: LongDateTime,
    x_min: FWord,
    y_min: FWord,
    x_max: FWord,
    y_max: FWord,
    mac_style: MacStyle,

    /// Smallest readable size in pixels
    lowest_rec_ppem: u16,

    font_direction_hint: i16,
    glyph_data_format: i16,
}

struct HeadFlags(u16);
struct MacStyle(u16);
