use crate::{
    error::PdfResult,
    postscript::PostScriptLexer,
    stream::{decode_stream, Stream},
    Resolve,
};

/// A type 4 function (PDF 1.3), also called a PostScript calculator function, shall be
/// represented as a stream containing code written in a small subset of the PostScript language
#[derive(Debug)]
pub struct PostScriptCalculatorFunction {
    // todo: probably want to actually store the AST instead of just the lexer
    tokens: PostScriptLexer,
}

impl PostScriptCalculatorFunction {
    pub fn from_stream(stream: Stream, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let buffer = decode_stream(&stream.stream, &stream.dict, resolver)?;

        Ok(Self {
            tokens: PostScriptLexer::new(buffer.into_boxed_slice()),
        })
    }
}
