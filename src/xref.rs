use std::collections::HashMap;

use crate::{Lex, ParseError, PdfResult, Reference};

const START_XREF_SIGNATURE: &[u8; 9] = b"startxref";
const KILOBYTE: usize = 1024;

/// The cross-reference table contains information
/// that permits random access to indirect objects
/// within the file so that the entire file need
/// not be read to locate any particular object
#[derive(Debug)]
pub struct Xref {
    pub objects: HashMap<usize, XrefEntry>,
}

impl Xref {
    pub fn get_offset(&self, reference: Reference) -> Option<usize> {
        self.objects
            .get(&reference.object_number)
            .map(|entry| entry.byte_offset)
    }
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

pub(crate) struct XrefParser<'a> {
    file: &'a [u8],
    pos: usize,
}

impl Lex for XrefParser<'_> {
    fn buffer(&self) -> &[u8] {
        &self.file
    }

    fn cursor(&self) -> usize {
        self.pos
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.pos
    }
}

impl<'a> XrefParser<'a> {
    pub fn new(file: &'a [u8]) -> Self {
        Self { file, pos: 0 }
    }

    /// We read backwards in 1024 byte chunks, looking for `"startxref"`
    pub fn read_xref(&mut self) -> PdfResult<(Xref, usize)> {
        let mut pos = self.file.len().saturating_sub(1);

        let idx = loop {
            if pos == 0 {
                todo!();
            }

            let next_pos = pos.saturating_sub(KILOBYTE - START_XREF_SIGNATURE.len());
            if let Some(start) = self.file[next_pos..=pos]
                .windows(START_XREF_SIGNATURE.len())
                .position(|window| window == START_XREF_SIGNATURE)
            {
                break start + next_pos;
            }

            pos = next_pos;
        };

        self.pos = idx;

        self.expect_byte(b's')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'a')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b't')?;
        self.expect_byte(b'x')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'f')?;

        self.skip_whitespace();

        let xref_pos = self.lex_whole_number().parse::<usize>().unwrap();

        self.pos = xref_pos;

        self.lex_xref()
    }

    fn lex_xref(&mut self) -> PdfResult<(Xref, usize)> {
        self.expect_byte(b'x')?;
        self.expect_byte(b'r')?;
        self.expect_byte(b'e')?;
        self.expect_byte(b'f')?;

        self.skip_whitespace();

        let mut objects = HashMap::new();

        loop {
            let idx_offset = self.lex_whole_number().parse::<usize>().unwrap();
            self.skip_whitespace();

            let num_of_entries = self.lex_whole_number().parse::<usize>().unwrap();
            self.expect_eol()?;

            objects.reserve(num_of_entries);

            for i in 0..num_of_entries {
                let byte_offset = self.lex_whole_number().parse::<usize>().unwrap();
                self.skip_whitespace();
                let generation = self.lex_whole_number().parse::<u16>().unwrap();
                self.skip_whitespace();
                let entry_kind = EntryKind::from_byte(self.next_byte_err()?)?;
                self.skip_whitespace();

                objects.insert(
                    i + idx_offset,
                    XrefEntry {
                        byte_offset,
                        generation,
                        kind: entry_kind,
                    },
                );
            }

            match self.peek_byte() {
                Some(b't') => break,
                Some(b'0'..=b'9') => continue,
                found => {
                    return Err(ParseError::MismatchedByteMany {
                        found,
                        expected: &[
                            b't', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
                        ],
                    })
                }
            }
        }

        Ok((Xref { objects }, self.pos))
    }
}
