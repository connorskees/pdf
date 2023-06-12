pub(crate) use {
    error::{PostScriptError, PostScriptResult},
    interpreter::PostscriptInterpreter,
};

// todo: standardize capitalization to Postscript or PostScript (presumably the latter)

mod builtin;
pub mod charstring;
mod decode;
mod error;
pub mod font;
mod graphics_operator;
mod interpreter;
mod lexer;
mod object;
mod operator;
