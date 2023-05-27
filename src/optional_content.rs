use crate::objects::Object;

#[derive(Debug, Clone, FromObj)]
pub struct OptionalContent;

#[derive(Debug, FromObj)]
pub struct OptionalContentProperties<'a> {
    /// An array of indirect references to all the optional content groups in the
    /// document, in any order. Every optional content group shall be included
    /// in this array.
    #[field("OCGs")]
    optional_content_groups: Vec<Object<'a>>,

    /// The default viewing optional content configuration dictionary
    #[field("D")]
    default_config: OptionalContentConfiguration<'a>,

    /// An array of alternate optional content configuration dictionaries
    #[field("Configs")]
    alternate_configs: Option<Vec<OptionalContentConfiguration<'a>>>,
}

#[derive(Debug, FromObj)]
struct OptionalContentConfiguration<'a> {
    /// A name for the configuration, suitable for presentation in a user interface.
    #[field("Name")]
    name: Option<String>,

    /// Name of the application or feature that created this configuration dictionary.
    #[field("Creator")]
    creator: Option<String>,

    /// Used to initialize the states of all the optional content groups in a
    /// document when this configuration is applied. The value of this entry
    /// shall be one of the following names:
    ///
    /// ON          The states of all groups shall be turned ON.
    /// OFF         The states of all groups shall be turned OFF.
    /// Unchanged   The states of all groups shall be left unchanged.
    ///
    /// After this initialization, the contents of the ON and OFF arrays shall
    /// be processed, overriding the state of the groups included in the arrays.
    ///
    /// Default value: ON.
    ///
    /// If BaseState is present in the document’s default configuration dictionary,
    /// its value shall be ON.
    #[field("BaseState")]
    base_state: Option<OptionalContentBaseState>,

    /// An array of optional content groups whose state shall be set to ON when
    /// this configuration is applied.
    ///
    /// If the BaseState entry is ON, this entry is redundant.
    // todo: Vec<OptionalContentGroup>
    #[field("ON")]
    on: Option<Vec<Object<'a>>>,

    /// An array of optional content groups whose state shall be set to OFF when
    /// this configuration is applied.
    ///
    /// If the BaseState entry is OFF, this entry is redundant.
    // todo: Vec<OptionalContentGroup>
    #[field("OFF")]
    off: Option<Vec<Object<'a>>>,

    /// A single intent name or an array containing any combination of names. It
    /// shall be used to determine which optional content groups’ states to consider
    /// and which to ignore in calculating the visibility of content
    ///
    /// PDF defines two intent names, View and Design. In addition, the name All
    /// shall indicate the set of all intents, including those not yet defined.
    ///
    /// Default value: View.
    ///
    /// The value shall be View for the document’s default configuration.
    #[field("Intent")]
    intent: Option<Intent>,

    /// An array of usage application dictionaries specifying which usage dictionary
    /// categories shall be consulted by conforming readers to automatically set
    /// the states of optional content groups based on external factors, such as
    /// the current system language or viewing magnification, and when they shall
    /// be applied.
    // todo: Vec<OptionalContentUsageApplication>
    #[field("AS")]
    applications: Option<Vec<Object<'a>>>,

    /// An array specifying the order for presentation of optional content groups
    /// in a conforming reader’s user interface. The array elements may include
    /// the following objects:
    ///
    /// Optional content group dictionaries, whose Name entry shall be displayed
    /// in the user interface by the conforming reader.
    ///
    /// Arrays of optional content groups which may be displayed by a conforming
    /// reader in a tree or outline structure. Each nested array may optionally
    /// have as its first element a text string to be used as a non-selectable
    /// label in a conforming reader’s user interface.
    ///
    /// Text labels in nested arrays shall be used to present collections of related
    /// optional content groups, and not to communicate actual nesting of content
    /// inside multiple layers of groups. To reflect actual nesting of groups in
    /// the content, such as for layers with sublayers, nested arrays of groups
    /// without a text label shall be used.
    ///
    /// An empty array [] explicitly specifies that no groups shall be presented.
    ///
    /// In the default configuration dictionary, the default value shall be an
    /// empty array; in other configuration dictionaries, the default shall be
    /// the Order value from the default configuration dictionary.
    ///
    /// Any groups not listed in this array shall not be presented in any user
    /// interface that uses the configuration.
    // todo: Vec<OptionalContentGroup>
    #[field("Order")]
    order: Option<Vec<Object<'a>>>,

    /// A name specifying which optional content groups in the Order array shall
    /// be displayed to the user.
    #[field("ListMode")]
    list_mode: Option<ListMode>,

    /// An array consisting of one or more arrays, each of which represents a
    /// collection of optional content groups whose states shall be intended to
    /// follow a radio button paradigm. That is, the state of at most one optional
    /// content group in each array shall be ON at a time. If one group is turned
    /// ON, all others shall be turned OFF. However, turning a group from ON to
    /// OFF does not force any other group to be turned ON.
    ///
    /// An empty array [] explicitly indicates that no such collections exist.
    ///
    /// In the default configuration dictionary, the default value shall be an
    /// empty array; in other configuration dictionaries, the default is the
    /// RBGroups value from the default configuration dictionary.
    // todo: better type
    #[field("RBGroups")]
    rb_groups: Option<Vec<Object<'a>>>,

    /// An array of optional content groups that shall be locked when this
    /// configuration is applied. The state of a locked group cannot be changed
    /// through the user interface of a conforming reader. Conforming writers
    /// can use this entry to prevent the visibility of content that depends on
    /// these groups from being changed by users.
    ///
    /// Default value: an empty array.
    ///
    /// A conforming reader may allow the states of optional content groups from
    /// being changed by means other than the user interface, such as JavaScript
    /// or items in the AS entry of a configuration dictionary.
    // todo: Vec<OptionalContentGroup>
    #[field("Locked")]
    locked: Option<Vec<Object<'a>>>,
}

#[derive(Debug, Clone, FromObj)]
pub struct OptionalContentGroup;

#[derive(Debug)]
struct OptionalContentUsage;
#[derive(Debug)]
struct OptionalContentUsageApplication;

#[pdf_enum]
enum ListMode {
    /// Display all groups in the Order array.
    AllPages = "AllPages",

    /// Display only those groups in the Order array that are referenced by
    /// one or more visible pages.
    VisiblePages = "VisiblePages",
}

impl Default for ListMode {
    fn default() -> Self {
        Self::AllPages
    }
}

#[pdf_enum]
enum OptionalContentBaseState {
    On = "ON",
    Off = "OFF",
    Unchanged = "Unchanged",
}

impl Default for OptionalContentBaseState {
    fn default() -> Self {
        Self::On
    }
}

#[pdf_enum]
enum Intent {
    /// Used for interactive use by document consumers
    View = "View",

    /// Used to represent a document designer’s structural organization of artwork,
    Design = "Design",

    /// Indicates the set of all intents, including those not yet defined
    All = "All",
}

impl Default for Intent {
    fn default() -> Self {
        Self::View
    }
}
