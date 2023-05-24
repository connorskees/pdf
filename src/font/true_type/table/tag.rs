use std::fmt::{self, Write};

#[derive(PartialEq, Eq)]
pub struct TableTag([u8; 4]);

impl TableTag {
    pub const fn new(tag: [u8; 4]) -> Self {
        Self(tag)
    }
}

impl fmt::Debug for TableTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.0[0] as char)?;
        f.write_char(self.0[1] as char)?;
        f.write_char(self.0[2] as char)?;
        f.write_char(self.0[3] as char)?;

        Ok(())
    }
}
