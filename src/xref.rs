use std::collections::HashMap;

use crate::{ParseError, PdfResult};

/// The cross-reference table contains information
/// that permits random access to indirect objects
/// within the file so that the entire file need
/// not be read to locate any particular object
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
