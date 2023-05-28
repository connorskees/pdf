use core::fmt;

#[derive(Debug)]
pub enum PdfRenderError {
    StackUnderflow,
}

impl fmt::Display for PdfRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for PdfRenderError {}
