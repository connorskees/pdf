use crate::{error::PdfResult, ParseError};

#[derive(Debug, Clone)]
pub(crate) enum StateModel {
    Marked(MarkedState),
    Review(ReviewState),
}

impl StateModel {
    pub fn default(state_model: &str) -> PdfResult<Self> {
        Ok(match state_model {
            "Marked" => Self::Marked(MarkedState::default()),
            "Review" => Self::Review(ReviewState::default()),
            found => {
                anyhow::bail!(ParseError::UnrecognizedVariant {
                    ty: "StateModel",
                    found: found.to_owned(),
                })
            }
        })
    }

    pub fn with_state(state_model: &str, state: &str) -> PdfResult<Self> {
        let state_model = Self::default(state_model)?;

        Ok(match state_model {
            Self::Marked(..) => Self::Marked(MarkedState::from_str(state)?),
            Self::Review(..) => Self::Review(ReviewState::from_str(state)?),
        })
    }
}

#[pdf_enum]
#[derive(Default)]
pub(crate) enum MarkedState {
    /// The annotation has been marked by the user
    Marked = "Marked",

    /// The annotation has not been marked by the user (the default)
    #[default]
    Unmarked = "Unmarked",
}

#[pdf_enum]
#[derive(Default)]
pub(crate) enum ReviewState {
    /// The user agrees with the change
    Accepted = "Accepted",

    /// The user disagrees with the change
    Rejected = "Rejected",

    /// The change has been cancelled
    Cancelled = "Cancelled",

    /// The change has been completed
    Completed = "Completed",

    /// The user has indicated nothing about the change (the default)
    #[default]
    None = "None",
}
