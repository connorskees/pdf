use super::TableTag;

#[derive(Debug)]
pub struct GlyfTable {
    pub(crate) glyphs: Vec<TrueTypeGlyph>,
}

impl GlyfTable {
    pub const TAG: TableTag = TableTag::new(*b"glyf");
}

#[derive(Debug)]
pub struct GlyphDescription {
    /// If the number of contours is positive or zero, it is a single glyph;
    /// If the number of contours less than zero, the glyph is compound
    number_of_contours: i16,

    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

#[derive(Debug)]
pub struct SimpleGlyph {
    /// Array of last points of each contour; array entries are point indices
    // todo: we should be able to get away with either `&[u16]` or `&[u8] here
    pub(crate) end_points_of_contours: Vec<u16>,

    pub(crate) instructions: Vec<u8>,
    pub(crate) flags: Vec<u8>,

    /// Array of x-coordinates; the first is relative to (0,0), others are relative
    /// to previous point
    pub(crate) x_coords: Vec<i16>,

    /// Array of y-coordinates; the first is relative to (0,0), others are relative
    /// to previous point
    pub(crate) y_coords: Vec<i16>,
}

impl SimpleGlyph {
    pub fn empty() -> Self {
        Self {
            end_points_of_contours: Vec::new(),
            instructions: Vec::new(),
            flags: Vec::new(),
            x_coords: Vec::new(),
            y_coords: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum TrueTypeGlyph {
    Simple(SimpleGlyph),
    Compound(Vec<CompoundGlyphPartDescription>),
}

#[derive(Debug)]
pub struct OutlineFlag(u8);

impl OutlineFlag {
    /// If set, the point is on the curve; otherwise, it is off the curve
    pub const ON_CURVE: u8 = 1 << 0;

    /// If set, the corresponding x-coordinate is 1 byte long; Otherwise, the
    /// corresponding x-coordinate is 2 bytes long
    pub const X_SHORT_VECTOR: u8 = 1 << 1;

    /// If set, the corresponding y-coordinate is 1 byte long; Otherwise, the
    /// corresponding y-coordinate is 2 bytes long
    pub const Y_SHORT_VECTOR: u8 = 1 << 2;

    /// If set, the next byte specifies the number of additional times this set
    /// of flags is to be repeated. In this way, the number of flags listed can
    /// be smaller than the number of points in a character.
    pub const REPEAT: u8 = 1 << 3;

    /// This flag has one of two meanings, depending on how the x-Short Vector
    /// flag is set.
    ///
    /// If the x-Short Vector bit is set, this bit describes the sign of the
    /// value, with a value of 1 equalling positive and a zero value negative.
    ///
    /// If the x-short Vector bit is not set, and this bit is set, then the
    /// current x-coordinate is the same as the previous x-coordinate.
    ///
    /// If the x-short Vector bit is not set, and this bit is not set, the current
    /// x-coordinate is a signed 16-bit delta vector. In this case, the delta
    /// vector is the change in x
    pub const X_SAME_OR_POSITIVE: u8 = 1 << 4;
    pub const Y_SAME_OR_POSITIVE: u8 = 1 << 5;
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CompoundGlyphComponentFlags(u16);

impl CompoundGlyphComponentFlags {
    pub const ARG_1_AND_2_ARE_WORDS: u16 = 1 << 0;
    pub const ARGS_ARE_XY_VALUES: u16 = 1 << 1;
    pub const ROUND_XY_TO_GRID: u16 = 1 << 2;
    pub const WE_HAVE_A_SCALE: u16 = 1 << 3;
    pub const _OBSOLETE: u16 = 1 << 4;
    pub const MORE_COMPONENTS: u16 = 1 << 5;
    pub const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 1 << 6;
    pub const WE_HAVE_A_TWO_BY_TWO: u16 = 1 << 7;
    pub const WE_HAVE_INSTRUCTIONS: u16 = 1 << 8;
    pub const USE_MY_METRICS: u16 = 1 << 9;
    pub const OVERLAP_COMPOUND: u16 = 1 << 10;
}

#[derive(Debug)]
enum CompoundTransformationOption {
    One,
    Two,
    Three,
}
