use std::{
    io,
    num::{ParseIntError, TryFromIntError},
};

use crate::{
    objects::{Object, ObjectType},
    postscript::PostScriptError,
    render::error::PdfRenderError,
};

#[derive(Debug)]
pub enum ParseError {
    MismatchedByte {
        expected: u8,
        found: Option<u8>,
    },
    MismatchedByteMany {
        expected: &'static [u8],
        found: Option<u8>,
    },
    UnexpectedEof,
    IoError(io::Error),
    MismatchedObjectType {
        expected: ObjectType,
        // found: Object,
    },
    MismatchedObjectTypeAny {
        expected: &'static [ObjectType],
        // found: Object,
    },
    InvalidDictionaryValueForKey {
        key: &'static str,
        // found: Object,
    },
    MissingRequiredKey {
        key: &'static str,
    },
    ArrayOfInvalidLength {
        expected: usize,
        // found: Vec<Object>,
    },
    UnrecognizedVariant {
        found: String,
        ty: &'static str,
    },
    MismatchedTypeKey {
        expected: &'static str,
        found: String,
    },
    IntegerConversionError(TryFromIntError),
    ParseIntegerError(ParseIntError),
    PostScriptError(PostScriptError),
    RenderError(PdfRenderError),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<PdfRenderError> for ParseError {
    fn from(err: PdfRenderError) -> Self {
        Self::RenderError(err)
    }
}

impl From<TryFromIntError> for ParseError {
    fn from(err: TryFromIntError) -> Self {
        Self::IntegerConversionError(err)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        Self::ParseIntegerError(err)
    }
}

pub type PdfResult<T> = Result<T, ParseError>;
