use std::{borrow::Cow, collections::HashMap};

use crate::{
    catalog::Trailer,
    flate_decoder::{FlateDecoder, FlateDecoderParams},
    objects::{Dictionary, Object, ObjectType},
    xref::{
        stream::{XrefStream, XrefStreamDict},
        Xref,
    },
    Lex, ParseError, PdfResult, Reference, Resolve, TypeOrArray,
};

use super::{stream::parser::XrefStreamParser, XrefEntry};

const START_XREF_SIGNATURE: &[u8; 9] = b"startxref";
const KILOBYTE: usize = 1024;

#[derive(Debug)]
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

    fn lex_dict(&mut self) -> PdfResult<Object> {
        Ok(Object::Dictionary(self.lex_dict_ignore_stream()?))
    }
}

impl<'a> Resolve for XrefParser<'a> {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object> {
        Ok(Object::Reference(reference))
    }
}

pub(crate) enum TrailerOrOffset {
    Trailer(Trailer),
    Offset(usize),
}

// todo: do this better
pub(crate) struct XrefAndTrailer {
    pub(crate) xref: Xref,
    pub(crate) trailer_or_offset: TrailerOrOffset,
}

impl<'a> XrefParser<'a> {
    pub fn new(file: &'a [u8]) -> Self {
        Self { file, pos: 0 }
    }

    /// We read backwards in 1024 byte chunks, looking for `"startxref"`
    pub fn read_xref(&mut self) -> PdfResult<XrefAndTrailer> {
        let mut pos = self.file.len().saturating_sub(1);

        let idx = loop {
            if pos == 0 {
                todo!();
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

        self.pos = xref_pos;

        self.lex_xref()
    }

    fn lex_xref_stream(&mut self) -> PdfResult<XrefAndTrailer> {
        // todo: lex external object instead of ad hoc+throwing away nums
        self.lex_whole_number();
        self.skip_whitespace();
        self.lex_whole_number();
        self.skip_whitespace();
        self.expect_bytes(b"obj")?;
        self.skip_whitespace();

        let xref_stream_dict = match self.lex_object()? {
            Object::Dictionary(dict) => XrefStreamDict::from_dict(dict, self)?,
            obj => {
                return Err(ParseError::MismatchedObjectType {
                    expected: ObjectType::Dictionary,
                    found: obj,
                })
            }
        };

        let stream = self.lex_stream(xref_stream_dict)?;

        let params = FlateDecoderParams::from_dict(
            match stream.dict.stream_dict.decode_parms {
                Some(TypeOrArray::Type(t)) => t,
                None => Dictionary::new(HashMap::new()),
                params => todo!("{:?}", params),
            },
            self,
        )?;

        let decoded_stream = FlateDecoder::new(Cow::Owned(stream.stream), params).decode();

        let xref =
            XrefStreamParser::new(&decoded_stream, stream.dict.w, stream.dict.index).parse()?;

        self.skip_whitespace();
        self.expect_bytes(b"endobj")?;

        Ok(XrefAndTrailer {
            xref,
            trailer_or_offset: TrailerOrOffset::Trailer(stream.dict.trailer),
        })
    }

    fn lex_stream(&mut self, stream_dict: XrefStreamDict) -> PdfResult<XrefStream> {
        self.expect_bytes(b"stream")?;
        self.expect_eol()?;

        let stream =
            self.get_byte_range(self.cursor(), self.cursor() + stream_dict.stream_dict.len);
        *self.cursor_mut() += stream_dict.stream_dict.len;

        self.expect_eol()?;

        self.expect_bytes(b"endstream")?;
        self.expect_eol()?;

        Ok(XrefStream {
            stream,
            dict: stream_dict,
        })
    }

    fn lex_xref(&mut self) -> PdfResult<XrefAndTrailer> {
        if !self.next_matches(b"xref") {
            return self.lex_xref_stream();
        }

        self.expect_bytes(b"xref")?;

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
                            return Err(ParseError::MismatchedByteMany {
                                expected: &[b'f', b'n'],
                                found: Some(entry_kind),
                            })
                        }
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

        Ok(XrefAndTrailer {
            xref: Xref { objects },
            trailer_or_offset: TrailerOrOffset::Offset(self.pos),
        })
    }
}
