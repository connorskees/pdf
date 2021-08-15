use std::{borrow::Cow, collections::HashMap, convert::TryFrom};

use crate::{
    error::PdfResult,
    lex::{LexBase, LexObject},
    objects::{Dictionary, Object, Reference},
    stream::StreamDict,
    Resolve,
};

#[derive(Debug)]
pub(crate) struct ObjectStream<'a> {
    pub(crate) stream: Cow<'a, [u8]>,
    pub(crate) dict: ObjectStreamDict,
}

#[derive(Debug)]
pub(crate) struct ObjectStreamDict {
    pub(crate) stream_dict: StreamDict,

    /// The number of indirect objects stored in the stream
    pub(crate) n: usize,

    /// The byte offset in the decoded stream of the first compressed object
    pub(crate) first: usize,

    /// A reference to another object stream, of which the current object stream
    /// shall be considered an extension
    ///
    /// Both streams are considered part of a collection of object streams.
    ///
    /// A given collection consists of a set of streams whose `Extends` links form a
    /// directed acyclic graph
    pub(crate) extends: Option<Reference>,
}

impl ObjectStreamDict {
    const TYPE: &'static str = "ObjStm";

    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, true)?;

        let n = usize::try_from(dict.expect_unsigned_integer("N", resolver)?)?;
        let first = usize::try_from(dict.expect_unsigned_integer("First", resolver)?)?;

        let extends = dict.get_reference("Extends")?;

        let stream_dict = StreamDict::from_dict(dict, resolver)?;

        debug_assert!(extends.is_none(), "todo: objstm extends");

        Ok(Self {
            stream_dict,
            n,
            first,
            extends,
        })
    }
}

#[derive(Debug)]
pub(crate) struct ObjectStreamParser {
    decoded_stream: Vec<u8>,
    cursor: usize,
    object_stream_dict: ObjectStreamDict,

    /// Map from object number to offset
    offsets: HashMap<usize, usize>,
}

impl ObjectStreamParser {
    pub fn new(decoded_stream: Vec<u8>, object_stream_dict: ObjectStreamDict) -> PdfResult<Self> {
        let mut parser = Self {
            decoded_stream,
            object_stream_dict,
            cursor: 0,
            offsets: HashMap::new(),
        };

        for _ in 0..parser.object_stream_dict.n {
            let object_number = parser.lex_whole_number().parse()?;
            parser.skip_whitespace();
            let byte_offset = parser.lex_whole_number().parse()?;
            parser.skip_whitespace();

            parser.offsets.insert(object_number, byte_offset);
        }

        parser
            .decoded_stream
            .drain(..parser.object_stream_dict.first)
            .for_each(drop);

        Ok(parser)
    }

    pub fn parse_object(&mut self, reference: Reference) -> PdfResult<Object> {
        let byte_offset = match self.offsets.get(&reference.object_number) {
            Some(&v) => v,
            None => return Ok(Object::Null),
        };

        self.cursor = byte_offset;

        self.lex_object()
    }
}

impl<'a> LexBase<'a> for ObjectStreamParser {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.cursor
    }

    fn buffer(&self) -> &[u8] {
        &self.decoded_stream
    }
}

impl<'a> LexObject<'a> for ObjectStreamParser {
    fn lex_dict(&mut self) -> PdfResult<Object> {
        let dict = self.lex_dict_ignore_stream()?;

        if self.next_matches(b"stream") {
            todo!()
        }

        Ok(Object::Dictionary(dict))
    }
}
