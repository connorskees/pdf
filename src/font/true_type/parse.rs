pub(crate) struct TrueTypeParser<'a> {
    buffer: &'a [u8],
    cursor: usize,
}

struct TableDirectory {
    offset: u32,
}

struct OffsetSubtable {
    number_of_tables: u16,

    /// the largest power of two less than or equal to the number of items in
    /// the table, i.e. the largest number of items that can be easily searched
    search_range: u16,

    /// log2(maximum power of 2 <= numTables)
    entry_selector: u16,

    /// numTables * 16 - searchRange
    range_shift: u16,
}

impl<'a> TrueTypeParser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn next(&mut self) -> Option<u8> {
        self.buffer.get(self.cursor).map(|b| {
            self.cursor += 1;
            *b
        })
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        let b1 = self.next()?;
        let b2 = self.next()?;

        Some(u16::from_be_bytes([b1, b2]))
    }

    pub fn read_u32_bytes(&mut self) -> Option<[u8; 4]> {
        Some(self.read_u32()?.to_be_bytes())
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        let b1 = self.next()?;
        let b2 = self.next()?;
        let b3 = self.next()?;
        let b4 = self.next()?;

        Some(u32::from_be_bytes([b1, b2, b3, b4]))
    }

    fn read_table_directory(&mut self) -> () {}

    fn read_offset_subtable(&mut self) -> Option<OffsetSubtable> {
        let _sfnt_version = self.read_u32()?;
        let number_of_tables = self.read_u16()?;
        let search_range = self.read_u16()?;
        let entry_selector = self.read_u16()?;
        let range_shift = self.read_u16()?;

        Some(OffsetSubtable {
            number_of_tables,
            search_range,
            entry_selector,
            range_shift,
        })
    }

    // fn parse_table(&mut self) ->
}
