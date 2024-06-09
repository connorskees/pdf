use crate::stream::Stream;

// todo: rename file? to_unicode.rs

#[derive(Debug, FromObj, Clone)]
#[obj_type("CMap")]
pub struct ToUnicodeCmapStream<'a> {
    #[field]
    stream: Stream<'a>,
}
