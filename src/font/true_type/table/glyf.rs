use super::TableTag;

pub struct GlyfTable {
    pub(crate) glyphs: Vec<Glyph>,
}

impl GlyfTable {
    pub const TAG: TableTag = TableTag::new(*b"glyf");
}

pub struct GlyphDescription {
    /// If the number of contours is positive or zero, it is a single glyph;
    /// If the number of contours less than zero, the glyph is compound
    number_of_contours: i16,

    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

pub struct SimpleGlyph {
    /// Array of last points of each contour; array entries are point indices
    // todo: we should be able to get away with either `&[u16]` or `&[u8] here
    pub(crate) end_points_of_contours: Vec<u16>,

    pub(crate) instructions: Vec<u8>,
    pub(crate) flags: Vec<u8>,

    /// Array of x-coordinates; the first is relative to (0,0), others are relative
    /// to previous point
    pub(crate) x_coords: Vec<u16>,

    /// Array of y-coordinates; the first is relative to (0,0), others are relative
    /// to previous point
    pub(crate) y_coords: Vec<u16>,
}

pub enum Glyph {
    Simple(SimpleGlyph),
    Compound(Vec<CompoundGlyphPartDescription>),
}

struct OutlineFlag(u8);

impl OutlineFlag {
    const ON_CURVE: u8 = 1 << 0;
    const X_SHORT_VECTOR: u8 = 1 << 1;
    const Y_SHORT_VECTOR: u8 = 1 << 2;
    const REPEAT: u8 = 1 << 3;
    const POSITIVE_X_SHORT_VECTOR: u8 = 1 << 4;
    const POSITIVE_Y_SHORT_VECTOR: u8 = 1 << 5;
}

pub struct CompoundGlyphPartDescription {
    flags: CompoundGlyphComponentFlags,

    /// Glyph index of component
    glyph_index: u16,

    /// X-offset for component or point number; type depends on bits 0 and 1 in component flags
    argument_one: u16,

    /// Y-offset for component or point number type depends on bits 0 and 1 in component flags
    argument_two: u16,

    transformation_option: CompoundTransformationOption,
}

struct CompoundGlyphComponentFlags(u16);

impl CompoundGlyphComponentFlags {
    const ARG_1_AND_2_ARE_WORDS: u16 = 1 << 0;
    const ARGS_ARE_XY_VALUES: u16 = 1 << 1;
    const ROUND_XY_TO_GRID: u16 = 1 << 2;
    const WE_HAVE_A_SCALE: u16 = 1 << 3;
    const _OBSOLETE: u16 = 1 << 4;
    const MORE_COMPONENTS: u16 = 1 << 5;
    const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 1 << 6;
    const WE_HAVE_A_TWO_BY_TWO: u16 = 1 << 7;
    const WE_HAVE_INSTRUCTIONS: u16 = 1 << 8;
    const USE_MY_METRICS: u16 = 1 << 9;
    const OVERLAP_COMPOUND: u16 = 1 << 10;
}

enum CompoundTransformationOption {
    One,
    Two,
    Three,
}
