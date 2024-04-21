use std::collections::HashMap;

use crate::{PdfResult, Reference};

pub(crate) use parser::{TrailerOrOffset, XrefParser};

mod parser;
pub mod stream;

/// The cross-reference table contains information
/// that permits random access to indirect objects
/// within the file so that the entire file need
/// not be read to locate any particular object
#[derive(Debug, Clone)]
pub struct Xref {
    // todo: map by generation AND object (reference)
    pub(crate) objects: HashMap<usize, XrefEntry>,
}

#[derive(Debug)]
pub enum ByteOffset {
    MainFile(usize),
    ObjectStream { byte_offset: usize, index: usize },
}

impl Xref {
    pub fn get_offset(&self, reference: Reference) -> PdfResult<Option<ByteOffset>> {
        Ok(
            if let Some(entry) = self.objects.get(&reference.object_number) {
                match entry {
                    XrefEntry::Free { .. } | XrefEntry::Null => None,
                    XrefEntry::InUse { byte_offset, .. } => {
                        Some(ByteOffset::MainFile(*byte_offset))
                    }
                    &XrefEntry::Compressed {
                        object_number,
                        index,
                    } => {
                        let byte_offset = match self.get_offset(Reference {
                            object_number: usize::try_from(object_number)?,
                            generation: 0,
                        })? {
                            Some(ByteOffset::MainFile(v)) => v,
                            Some(ByteOffset::ObjectStream { .. }) => todo!(),
                            None => return Ok(None),
                        };

                        Some(ByteOffset::ObjectStream { byte_offset, index })
                    }
                }
            } else {
                None
            },
        )
    }

    pub fn merge_with_previous(&mut self, previous: Xref) {
        for (key, value) in previous.objects.into_iter() {
            self.objects.entry(key).or_insert(value);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum XrefEntry {
    InUse {
        byte_offset: usize,
        generation_number: u16,
    },
    Free {
        next_free_object: u64,
        generation_number: u16,
    },
    Compressed {
        /// The object number of the object stream in which this object is stored
        ///
        /// The generation number of the object stream shall be implicitly 0
        object_number: u64,

        /// The index of this object within the object stream
        index: usize,
    },
    Null,
}
