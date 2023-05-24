use crate::{destination::Destination, file_specification::FileSpecification};

/// A go-to action changes the view to a specified destination (page, location, and magnification factor)
#[derive(Debug, FromObj)]
pub struct GoToAction {
    /// The destination to jump to
    #[field("D")]
    d: Destination,
}

/// A remote go-to action is similar to an ordinary go-to action but jumps to a destination in
/// another PDF file instead of the current file
#[derive(Debug, FromObj)]
pub struct GoToRemoteAction<'a> {
    /// The file in which the destination shall be located
    #[field("F")]
    f: FileSpecification<'a>,

    /// The destination to jump to. If the value is an array defining an explicit destination, its
    /// first element shall be a page number within the remote document rather than an indirect reference
    /// to a page object in the current document.
    ///
    /// The first page shall be numbered 0.
    #[field("D")]
    d: Destination,

    /// A flag specifying whether to open the destination document in a new window.
    ///
    /// If this flag is false, the destination document replaces the current document in the same window.
    /// If this entry is absent, the conforming reader should behave in accordance with its preference
    #[field("NewWindow")]
    new_window: Option<bool>,
}
