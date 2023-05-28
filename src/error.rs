use std::{
    error::Error,
    fmt, io,
    num::{ParseIntError, TryFromIntError},
};

use crate::{objects::ObjectType, postscript::PostScriptError, render::error::PdfRenderError};

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
    },
    MismatchedObjectTypeAny {
        expected: &'static [ObjectType],
    },
    InvalidDictionaryValueForKey {
        key: &'static str,
    },
    MissingRequiredKey {
        key: &'static str,
    },
    ArrayOfInvalidLength {
        expected: usize,
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

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

pub type PdfResult<T> = anyhow::Result<T>;
