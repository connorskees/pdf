#[derive(Debug)]
pub struct NameTable {
    format: u16,
    string_offset: u16,
    name_records: Vec<NameRecord>,
    name: String,
}

#[derive(Debug)]
pub struct NameRecord {
    /// Platform identifier code.
    platform_id: u16,
    /// Platform-specific encoding identifier.
    platform_specific_id: u16,
    /// Language identifier.
    language_id: u16,
    /// Name identifier.
    name_id: u16,
    /// Name string length in bytes.
    length: u16,
    /// Name string offset in bytes from stringOffset.
    offset: u16,
}
