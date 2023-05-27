/// A URI action causes a URI to be resolved
#[derive(Debug, FromObj)]
pub struct UriAction {
    /// The uniform resource identifier to resolve, encoded in 7-bit ASCII
    // todo: perhaps ascii string as a newtype
    #[field("URI")]
    uri: String,

    /// A flag specifying whether to track the mouse position when the URI is
    /// resolved.
    ///
    /// Default value: false.
    ///
    /// This entry applies only to actions triggered by the user's clicking an
    /// annotation; it shall be ignored for actions associated with outline
    /// items or with a document's OpenAction entry.
    #[field("IsMap", default = false)]
    is_map: bool,
}
