use std::{borrow::Cow, fmt, num::ParseIntError};

use crate::error::ParseError;

use super::object::PostScriptString;

pub type PostScriptResult<T> = anyhow::Result<T>;

/*
postscript spec page 523

configurationerror setpagedevice or setdevparams request cannot be satisfied
dictfull No more room in dictionary
dictstackoverflow Too many begin operators
dictstackunderflow Too many end operators
execstackoverflow Executive stack nesting too deep
handleerror Called to report error information
interrupt External interrupt request (for example, Control-C)
invalidaccess Attempt to violate access attribute
invalidexit exit not in loop
invalidfileaccess Unacceptable access string
invalidfont Invalid Font resource name or font or CIDFont dictionary
invalidrestore Improper restore
ioerror Input/output error
limitcheck Implementation limit exceeded
nocurrentpoint Current point undefined
rangecheck Operand out of bounds
stackoverflow Operand stack overflow
stackunderflow Operand stack underflow
syntaxerror PostScript language syntax error
timeout Time limit exceeded
typecheck Operand of wrong type
undefined Name not known
undefinedfilename File not found
undefinedresource Resource instance not found
undefinedresult Overflow, underflow, or meaningless result
unmatchedmark Expected mark not on stack
unregistered Internal error
VMerror Virtual memory exhausted
*/

#[derive(Debug)]
pub enum PostScriptError {
    ParseError(Cow<'static, str>),
    ParseIntError(ParseIntError),
    ParseFloatError(fast_float::Error),

    /// No more room in dictionary
    DictionaryFull,

    /// Too many begin operators
    DictStackOverflow,

    /// Too many end operators
    DictStackUnderflow,

    /// Executive stack nesting too deep
    ExecStackOverflow,

    /// Operand stack overflow
    StackOverflow,

    /// Operand stack underflow
    StackUnderflow,

    /// Operand of wrong type
    TypeCheck,

    /// Operand out of bounds
    RangeCheck,

    /// Name not known
    Undefined {
        key: PostScriptString,
    },

    /// Invalid Font resource name or font or CIDFont dictionary
    InvalidFont,
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

impl From<PostScriptError> for ParseError {
    fn from(err: PostScriptError) -> Self {
        Self::PostScriptError(err)
    }
}

impl fmt::Display for PostScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for PostScriptError {}
