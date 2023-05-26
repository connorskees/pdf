#[derive(Debug, Clone)]
pub struct LocaTable {
    pub offsets: Vec<u32>,
}

impl LocaTable {
    pub fn get_glyf_entry(&self, char_code: u32) -> Option<GlyfEntry> {
        let offset = self.offsets[char_code as usize];
        let len = self.offsets[char_code as usize + 1] - offset;

        Some(GlyfEntry { offset, len })
    }
}

#[derive(Debug)]
pub struct GlyfEntry {
    pub offset: u32,
    pub len: u32,
}
