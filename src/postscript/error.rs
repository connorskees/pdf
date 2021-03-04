use std::{borrow::Cow, num::ParseIntError};

pub type PostScriptResult<T> = Result<T, PostScriptError>;

pub enum PostScriptError {
    ParseError(Cow<'static, str>),
    ParseIntError(ParseIntError),
    ParseFloatError(fast_float::Error),
}

impl From<ParseIntError> for PostScriptError {
    fn from(err: ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl From<fast_float::Error> for PostScriptError {
    fn from(err: fast_float::Error) -> Self {
        Self::ParseFloatError(err)
    }
}
