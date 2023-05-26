use crate::font::true_type::Fixed;

#[derive(Debug)]
pub struct MaxpTable {
    /// 0x00010000 (1.0)
    pub version: Fixed,
    /// the number of glyphs in the font
    pub num_glyphs: u16,
    /// points in non-compound glyph
    pub max_points: u16,
    /// contours in non-compound glyph
    pub max_contours: u16,
    /// points in compound glyph
    pub max_component_points: u16,
    /// contours in compound glyph
    pub max_component_contours: u16,
    /// set to 2
    pub max_zones: u16,
    /// points used in Twilight Zone (Z0)
    pub max_twilight_points: u16,
    /// number of Storage Area locations
    pub max_storage: u16,
    /// number of FDEFs
    pub max_function_defs: u16,
    /// number of IDEFs
    pub max_instruction_defs: u16,
    /// maximum stack depth
    pub max_stack_elements: u16,
    /// byte count for glyph instructions
    pub max_size_of_instructions: u16,
    /// number of glyphs referenced at top level
    pub max_component_elements: u16,
    /// levels of recursion, set to 0 if font has only simple glyphs
    pub max_component_depth: u16,
}

#[derive(Debug)]
pub struct MaxpPostscriptTable {
    /// 0x00005000 (0.5)
    pub version: Fixed,
    /// the number of glyphs in the font
    pub num_glyphs: u16,
}
