use super::TableTag;

#[derive(Debug)]
pub struct FontDirectory {
    pub offset_subtable: OffsetSubtable,
    pub table_directory: TableDirectory,
}

impl FontDirectory {
    pub fn find_table_offset(&self, tag: TableTag) -> Option<u32> {
        self.table_directory
            .0
            .iter()
            .find(|entry| entry.tag == tag)
            .map(|entry| entry.offset)
    }
}

#[derive(Debug)]
pub struct TableDirectory(pub Vec<DirectoryTableEntry>);

#[derive(Debug)]
pub struct DirectoryTableEntry {
    pub tag: TableTag,
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug)]
pub struct OffsetSubtable {
    pub number_of_tables: u16,

    /// the largest power of two less than or equal to the number of items in
    /// the table, i.e. the largest number of items that can be easily searched
    pub search_range: u16,

    /// log2(maximum power of 2 <= numTables)
    pub entry_selector: u16,

    /// numTables * 16 - searchRange
    pub range_shift: u16,
}
