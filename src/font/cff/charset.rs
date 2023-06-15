#[derive(Debug)]
pub enum CffCharset {
    IsoAdobe,
    Expert,
    ExpertSubset,
    Zero { name_ids: Vec<u16> },
    One(Vec<CharsetRangeOne>),
    Two(Vec<CharsetRangeTwo>),
}

#[derive(Debug)]
pub struct CharsetRangeOne {
    pub first: u16,
    /// Number of glyphs after `first`
    pub count: u8,
}

#[derive(Debug)]
pub struct CharsetRangeTwo {
    pub first: u16,
    /// Number of glyphs after `first`
    pub count: u16,
}
