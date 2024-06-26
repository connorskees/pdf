use std::{borrow::Cow, collections::HashMap};

use crate::{
    filter::decode_stream,
    lex::{LexBase, LexObject},
    objects::Object,
    trailer::Trailer,
    xref::{
        stream::{XrefStream, XrefStreamDict},
        Xref,
    },
    PdfResult, Reference, Resolve,
};

use super::{stream::parser::XrefStreamParser, XrefEntry};

const START_XREF_SIGNATURE: &[u8; 9] = b"startxref";
const KILOBYTE: usize = 1024;

#[derive(Debug)]
pub(crate) struct XrefParser {
    file: Vec<u8>,
    pos: usize,
}

impl<'a> LexBase<'a> for XrefParser {
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

impl<'a> LexObject<'a> for XrefParser {
    fn lex_dict(&mut self) -> PdfResult<Object<'a>> {
        Ok(Object::Dictionary(self.lex_dict_ignore_stream()?))
    }
}

impl<'a> Resolve<'a> for XrefParser {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object<'a>> {
        Ok(Object::Reference(reference))
    }

    fn reference_exists(&mut self, _reference: Reference) -> PdfResult<bool> {
        // todo: wrong! but i forget how xref parser works
        Ok(true)
    }
}

#[derive(Debug)]
pub(crate) enum TrailerOrOffset<'a> {
    Trailer(Trailer<'a>),
    Offset(usize),
}

// todo: do this better
#[derive(Debug)]
pub(crate) struct XrefAndTrailer<'a> {
    pub(crate) xref: Xref,
    pub(crate) trailer_or_offset: TrailerOrOffset<'a>,
}

impl<'a> XrefParser {
    pub fn new(file: Vec<u8>) -> Self {
        Self { file, pos: 0 }
    }

    /// We read backwards in 1024 byte chunks, looking for `"startxref"`
    pub fn read_xref(&mut self) -> PdfResult<XrefAndTrailer<'a>> {
        let mut pos = self.file.len().saturating_sub(1);

        let idx = loop {
            if pos == 0 {
                todo!("failed to find xref");
            }

            let next_pos = pos.saturating_sub(KILOBYTE - START_XREF_SIGNATURE.len());
            // todo: use rabin-karp or something similar
            if let Some(start) = self.file[next_pos..=pos]
                .windows(START_XREF_SIGNATURE.len())
                .position(|window| window == START_XREF_SIGNATURE)
            {
                break start + next_pos;
            }

            pos = next_pos;
        };

        self.pos = idx;

        self.expect_bytes(START_XREF_SIGNATURE)?;

        self.skip_whitespace();

        let xref_pos = self.lex_whole_number().parse::<usize>().unwrap();

        self.parse_xref_at_offset(xref_pos)
    }

    fn parse_xref_stream(&mut self, is_previous: bool) -> PdfResult<XrefAndTrailer<'a>> {
        self.read_obj_prelude()?;

        let xref_stream_dict = match self.lex_object()? {
            Object::Dictionary(dict) => XrefStreamDict::from_dict(dict, is_previous, self)?,
            obj => anyhow::bail!("expected dict, found {:?}", obj),
        };

        let stream = self.lex_stream(xref_stream_dict)?;
        let decoded_stream = decode_stream(&stream.stream, &stream.dict.stream_dict, self)?;

        let mut xref =
            XrefStreamParser::new(&decoded_stream, stream.dict.w, stream.dict.index).parse()?;

        self.read_obj_trailer()?;

        if !is_previous {
            let mut prev = stream.dict.trailer.prev;
            while let Some(prev_offset) = prev {
                self.pos = prev_offset;
                let xref_and_trailer = self.parse_xref_stream(true)?;

                xref.merge_with_previous(xref_and_trailer.xref);

                let prev_trailer = match xref_and_trailer.trailer_or_offset {
                    TrailerOrOffset::Trailer(trailer) => trailer,
                    TrailerOrOffset::Offset(..) => {
                        anyhow::bail!("can't parse literal trailer without lexer")
                    }
                };

                prev = prev_trailer.prev;
            }
        }

        Ok(XrefAndTrailer {
            xref,
            trailer_or_offset: TrailerOrOffset::Trailer(stream.dict.trailer),
        })
    }

    fn lex_stream(&mut self, stream_dict: XrefStreamDict<'a>) -> PdfResult<XrefStream<'a>> {
        self.expect_bytes(b"stream")?;
        self.expect_eol()?;

        let stream =
            self.get_byte_range(self.cursor(), self.cursor() + stream_dict.stream_dict.len);

        *self.cursor_mut() += stream_dict.stream_dict.len;

        self.expect_eol()?;
        self.skip_whitespace();

        self.expect_bytes(b"endstream")?;
        self.skip_whitespace();

        Ok(XrefStream {
            stream: Cow::Borrowed(stream),
            dict: stream_dict,
        })
    }

    pub fn parse_xref_at_offset(&mut self, offset: usize) -> PdfResult<XrefAndTrailer<'a>> {
        self.pos = offset;

        if !self.next_matches(b"xref") {
            return self.parse_xref_stream(false);
        }

        self.expect_bytes(b"xref")?;

        self.skip_whitespace();

        let mut objects = HashMap::new();

        loop {
            let idx_offset = self.lex_whole_number().parse::<usize>().unwrap();
            self.skip_whitespace();

            let num_of_entries = self.lex_whole_number().parse::<usize>().unwrap();
            self.skip_whitespace();

            objects.reserve(num_of_entries);

            for i in 0..num_of_entries {
                let byte_offset = self.lex_whole_number().parse::<usize>().unwrap();
                self.skip_whitespace();
                let generation_number = self.lex_whole_number().parse::<u16>().unwrap();
                self.skip_whitespace();
                let entry_kind = self.next_byte_err()?;
                self.skip_whitespace();

                objects.insert(
                    i + idx_offset,
                    match entry_kind {
                        b'f' => XrefEntry::Free {
                            next_free_object: byte_offset as u64,
                            generation_number,
                        },
                        b'n' => XrefEntry::InUse {
                            byte_offset,
                            generation_number,
                        },
                        _ => {
                            anyhow::bail!("expected `f` or `n`, found {:?}", char::from(entry_kind))
                        }
                    },
                );
            }

            match self.peek_byte() {
                Some(b't') => break,
                Some(b'0'..=b'9') => continue,
                found => {
                    anyhow::bail!(
                        "expected number (0-9) or `t`, found {:?}",
                        found.map(char::from)
                    )
                }
            }
        }

        Ok(XrefAndTrailer {
            xref: Xref { objects },
            trailer_or_offset: TrailerOrOffset::Offset(self.pos),
        })
    }
}
