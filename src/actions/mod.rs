use crate::{
    error::{ParseError, PdfResult},
    objects::{Object, ObjectType},
    FromObj, Resolve,
};

use self::goto::GoToRemoteAction;
pub use self::{goto::GoToAction, uri::UriAction};

mod goto;
mod uri;

#[derive(Debug, Clone)]
pub struct Actions<'a> {
    action: Action<'a>,

    /// The next action or sequence of actions that shall be performed after the action
    /// represented by this dictionary.
    ///
    /// The value is either a single action dictionary or an array of action dictionaries
    /// that shall be performed in order
    // todo: should this actually be Vec<Action>?
    next: Option<Vec<Self>>,
}

#[derive(Debug, Clone)]
enum Action<'a> {
    GoTo(GoToAction),
    GoToRemote(GoToRemoteAction<'a>),
    Uri(UriAction),
}

impl<'a> Actions<'a> {
    const TYPE: &'static str = "Action";

    fn maybe_array(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Vec<Self>> {
        Ok(match resolver.resolve(obj)? {
            Object::Array(arr) => arr
                .into_iter()
                .map(|obj| Actions::from_obj(obj, resolver))
                .collect::<PdfResult<Vec<Actions>>>()?,
            obj @ Object::Dictionary(..) => vec![Actions::from_obj(obj, resolver)?],
            _ => {
                anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Array, ObjectType::Dictionary],
                });
            }
        })
    }
}

impl<'a> FromObj<'a> for Actions<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut dict = resolver.assert_dict(obj)?;

        let action_type = ActionType::from_str(&dict.expect_name("S", resolver)?)?;

        let next = dict
            .get_object("Next", resolver)?
            .map(|obj| Actions::maybe_array(obj, resolver))
            .transpose()?;

        let action = match action_type {
            ActionType::GoTo => {
                Action::GoTo(GoToAction::from_obj(Object::Dictionary(dict), resolver)?)
            }
            ActionType::GoToRemote => Action::GoToRemote(GoToRemoteAction::from_obj(
                Object::Dictionary(dict),
                resolver,
            )?),
            ActionType::Uri => {
                Action::Uri(UriAction::from_obj(Object::Dictionary(dict), resolver)?)
            }
            _ => todo!(),
        };

        Ok(Self { action, next })
    }
}

#[pdf_enum]
pub enum ActionType {
    /// Go to a destination in the current document
    GoTo = "GoTo",

    /// Go to a destination in another document
    GoToRemote = "GoToR",

    /// Go to a destination in an embedded file
    GoToEmbedded = "GoToE",

    /// Launch an application, usually to open a file
    Launch = "Launch",

    /// Begin reading an article thread
    Thread = "Thread",

    /// Resolve a uniform resource identifier
    Uri = "URI",

    /// Play a sound
    Sound = "Sound",

    /// Play a movie
    Movie = "Movie",

    /// Set an annotation's Hidden flag
    Hide = "Hide",

    /// Execute an action predefined by the conforming reader
    Named = "Named",

    /// Send data to a uniform resource locator
    SubmitForm = "SubmitForm",

    /// Set fields to their default values
    ResetForm = "ResetForm",

    /// Import field values from a file
    ImportData = "ImportData",

    /// Execute a JavaScript script
    JavaScript = "JavaScript",

    /// Set the states of optional content groups
    ///
    /// NOTE: This action is considered obsolete and should not be used
    SetOptionalContentGroupState = "SetOCGState",

    /// Controls the playing of multimedia content
    Rendition = "Rendition",

    /// Updates the display of a document, using a transition dictionary
    Trans = "Trans",

    /// Set the current view of a 3D annotation
    GoTo3DView = "GoTo3DView",
}
