use std::collections::HashMap;

use crate::{ParseError, PdfResult};

#[derive(Debug)]
pub struct Xref {
    pub objects: HashMap<usize, XrefEntry>,
}

#[derive(Debug)]
pub struct XrefEntry {
    pub byte_offset: usize,
    pub generation: u16,
    pub kind: EntryKind,
}

#[derive(Debug)]
pub enum EntryKind {
    Free,
    InUse,
}

impl EntryKind {
    pub(crate) fn from_byte(b: u8) -> PdfResult<Self> {
        match b {
            b'f' => Ok(Self::Free),
            b'n' => Ok(Self::InUse),
            _ => Err(ParseError::MismatchedByteMany {
                expected: &[b'f', b'n'],
                found: Some(b),
            }),
        }
    }
}
