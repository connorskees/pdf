use crate::{error::PdfResult, objects::Dictionary, Resolve};

use super::state::StateModel;

/// A text annotation represents a "sticky note" attached to a point in the PDF document.
///
/// When closed, the annotation shall appear as an icon; when open, it shall display a pop-up
/// window containing the text of the note in a font and size chosen by the conforming reader.
/// Text annotations shall not scale and rotate with the page; they shall behave as if the
/// NoZoom and NoRotate annotation flags were always set
#[derive(Debug, Clone)]
pub(crate) struct TextAnnotation {
    /// A flag specifying whether the annotation shall initially be displayed open.
    ///
    /// Default value: false (closed).
    is_open: bool,

    /// The name of an icon that shall be used in displaying the annotation.
    ///
    /// Conforming readers shall provide predefined icon appearances for at least the
    /// following standard names:
    ///   * Comment
    ///   * Key
    ///   * Note
    ///   * Help
    ///   * NewParagraph
    ///   * Paragraph
    ///   * Insert
    ///
    /// Additional names may be supported as well.
    ///
    /// Default value: Note.
    name: TextAnnotationName,

    /// The state to which the original annotation shall be set
    state: Option<StateModel>,
}

#[derive(Debug, Clone)]
enum TextAnnotationName {
    Comment,
    Key,
    Note,
    Help,
    NewParagraph,
    Paragraph,
    Insert,
    Other(String),
}

impl TextAnnotationName {
    pub fn from_str(s: String) -> Self {
        match s.as_ref() {
            "Comment" => Self::Comment,
            "Key" => Self::Key,
            "Note" => Self::Note,
            "Help" => Self::Help,
            "NewParagraph" => Self::NewParagraph,
            "Paragraph" => Self::Paragraph,
            "Insert" => Self::Insert,
            _ => Self::Other(s),
        }
    }
}

impl Default for TextAnnotationName {
    fn default() -> Self {
        Self::Note
    }
}

impl TextAnnotation {
    const TYPE: &'static str = "Text";

    pub fn from_dict<'a>(
        dict: &mut Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let is_open = dict.get_bool("Open", resolver)?.unwrap_or(false);
        let name = dict
            .get_name("Name", resolver)?
            .map(TextAnnotationName::from_str)
            .unwrap_or_default();

        let state = if let Some(state) = dict.get_string("State", resolver)? {
            let state_model = dict.expect_string("StateModel", resolver)?;
            Some(StateModel::with_state(&state_model, &state)?)
        } else if let Some(state_model) = dict.get_string("StateModel", resolver)? {
            Some(StateModel::default(&state_model)?)
        } else {
            None
        };

        Ok(Self {
            is_open,
            name,
            state,
        })
    }
}
