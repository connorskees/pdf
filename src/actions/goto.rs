use crate::{
    catalog::Destination, error::PdfResult, file_specification::FileSpecification,
    objects::Dictionary, Resolve,
};

/// A go-to action changes the view to a specified destination (page, location, and magnification factor)
#[derive(Debug)]
pub struct GoToAction {
    /// The destination to jump to
    d: Destination,
}

impl GoToAction {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut impl Resolve) -> PdfResult<Self> {
        let d = Destination::from_arr(dict.expect_arr("D", resolver)?, resolver)?;

        Ok(Self { d })
    }
}

/// A remote go-to action is similar to an ordinary go-to action but jumps to a destination in
/// another PDF file instead of the current file
#[derive(Debug)]
pub struct GoToRemoteAction {
    /// The file in which the destination shall be located
    f: FileSpecification,

    /// The destination to jump to. If the value is an array defining an explicit destination, its
    /// first element shall be a page number within the remote document rather than an indirect reference
    /// to a page object in the current document.
    ///
    /// The first page shall be numbered 0.
    d: Destination,

    /// A flag specifying whether to open the destination document in a new window.
    ///
    /// If this flag is false, the destination document replaces the current document in the same window.
    /// If this entry is absent, the conforming reader should behave in accordance with its preference
    new_window: Option<bool>,
}

impl GoToRemoteAction {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut impl Resolve) -> PdfResult<Self> {
        let f = FileSpecification::from_obj(dict.expect_object("F", resolver)?, resolver)?;
        let d = Destination::from_arr(dict.expect_arr("D", resolver)?, resolver)?;
        let new_window = dict.get_bool("NewWindow", resolver)?;

        Ok(Self { f, d, new_window })
    }
}
