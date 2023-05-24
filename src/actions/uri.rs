use crate::{error::PdfResult, objects::Dictionary, Resolve};

/// A URI action causes a URI to be resolved
#[derive(Debug)]
pub struct UriAction {
    /// The uniform resource identifier to resolve, encoded in 7-bit ASCII
    // todo: perhaps ascii string as a newtype
    uri: String,

    /// A flag specifying whether to track the mouse position when the URI is
    /// resolved.
    ///
    /// Default value: false.
    ///
    /// This entry applies only to actions triggered by the user's clicking an
    /// annotation; it shall be ignored for actions associated with outline
    /// items or with a document's OpenAction entry.
    is_map: bool,
}

impl UriAction {
    pub fn from_dict<'a>(
        mut dict: Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let uri = dict.expect_string("URI", resolver)?;
        let is_map = dict.get_bool("IsMap", resolver)?.unwrap_or(false);

        Ok(Self { uri, is_map })
    }
}
