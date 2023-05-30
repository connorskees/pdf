use crate::stream::Stream;

#[derive(Debug, Clone, FromObj)]
pub struct PostScriptXObject<'a> {
    #[field]
    stream: Stream<'a>,

    /// A stream whose contents shall be used in place of the PostScript XObject's stream
    /// when the target PostScript interpreter is known to support only LanguageLevel 1
    #[field("Level1")]
    level_one: Option<Stream<'a>>,
}
