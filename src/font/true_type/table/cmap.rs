#[derive(Debug)]
pub struct CmapTable {
    /// Version number (Set to zero)
    version: u16,
    subtables: Vec<CmapSubtable>,
}

#[derive(Debug)]
pub enum CmapSubtable {
    Zero,
    Two,
    Four,
    Six,
    Eight,
    Ten,
    Twelve,
    Thirteen,
    Fourteen,
}
