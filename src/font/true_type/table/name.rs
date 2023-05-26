#[derive(Debug)]
pub struct NameTable {
    pub format: u16,
    pub string_offset: u16,
    pub name_records: Vec<NameRecord>,
}

#[derive(Debug)]
pub struct NameRecord {
    /// Platform identifier code.
    pub platform_id: u16,
    /// Platform-specific encoding identifier.
    pub platform_specific_id: u16,
    /// Language identifier.
    pub language_id: u16,
    /// Name identifier.
    pub name_id: u16,
    /// Name string length in bytes.
    pub length: u16,
    /// Name string offset in bytes from stringOffset.
    pub offset: u16,
    /// Offset in bytes from start of file
    pub absolute_offset: usize,
}
