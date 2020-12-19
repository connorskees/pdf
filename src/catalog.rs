/*
The catalog contains references to other objects defining
the document’s contents, outline, article threads, named
destinations, and other attributes.

In addition, it contains information about how the
document shall be displayed on the screen, such as
whether its outline and thumbnail page images shall
be displayed automatically and whether some location
other than the first page shall be shown when the
document is opened.
*/
use crate::{Name, ParseError, PdfResult};

/// See module level documentation
pub struct DocumentCatalog {
    // must be "Catalog"
    // TODO: definitely don't need this field
    ty: Name,

    /// The version of the PDF specification
    /// to which the document conforms(for example,
    /// 1.4) if later than the version specified in
    /// the file's header.
    ///
    /// If the header specifies a later version, or
    /// if this entry is absent, the document shall
    /// conform to the version specified in the header.
    /// This entry enables a conforming writer to
    /// update the version using an incremental update.
    ///
    /// The value of this entry shall be a name object,
    /// not a number, and therefore shall be preceded
    /// by a SOLIDUS (2Fh) character (/) when written
    /// in the PDF file (for example, /1.4).
    version: Option<Name>,

    extensions: Option<Extensions>,

    pages: Pages,

    /// A number tree defining the page labelling for
    /// the document. The keys in this tree shall be
    /// page indices; the corresponding values shall
    /// be page label dictionaries. Each page index
    /// shall denote the first page in a labelling
    /// range to which the specified page label dictionary
    /// applies. The tree shall include a value for
    /// page index 0.
    page_labels: Option<NumberTree>,

    names: Option<NameDictionary>,

    dests: Option<NamedDestinations>,

    // A viewer preferences dictionary specifying the way
    /// the document shall be displayed on the screen. If
    /// this entry is absent, conforming readers shall use
    /// their own current user preference settings.
    viewer_preferences: Option<ViewerPreferences>,

    page_layout: PageLayout,

    page_mode: PageMode,

    outlines: Option<DocumentOutline>,

    threads: Option<Vec<ThreadDictionary>>,

    /// A value specifying a destination that shall be displayed
    /// or an action that shall be performed when the document
    /// is opened. The value shall be either an array defining
    /// a destination or an action dictionary representing an action.
    /// If this entry is absent, the document shall be opened to the
    /// top of the first page at the default magnification factor.
    open_action: Option<OpenAction>,

    /// An additional-actions dictionary defining the actions
    /// that shall be taken in response to various trigger
    /// events affecting the document as a whole
    aa: Option<AdditionalActions>,

    /// A URI dictionary containing document-level information
    /// for URI actions
    uri: Option<UriDict>,

    acro_form: Option<AcroForm>,

    metadata: Option<MetadataStream>,

    struct_tree_root: Option<StructTreeRoot>,

    /// A mark information dictionary that shall contain information
    /// about the document's usage of Tagged PDF conventions
    mark_info: Option<MarkInfo>,

    /// A language identifier that shall specify the natural language
    /// for all text in the document except where overridden by language
    /// specifications for structure elements or marked content.
    ///
    /// If this entry is absent, the language shall be considered unknown.
    lang: Option<String>,

    /// A Web Capture information dictionary that shall contain state
    /// information used by any Web Capture extension
    spider_info: Option<WebCapture>,

    /// An array of output intent dictionaries that shall specify the colour
    /// characteristics of output devices on which the document might be rendered
    output_intents: Option<Vec<OutputIntent>>,

    piece_info: Option<PagePiece>,

    /// The document's optional content properties dictionary
    ///
    /// Required if a document contains optional content
    oc_properties: Option<OptionalContentProperties>,

    /// A permissions dictionary that shall specify user access permissions
    /// for the document.
    perms: Option<Permissions>,

    /// A dictionary that shall contain attestations regarding the content of a
    /// PDF document, as it relates to the legality of digital signatures
    legal: Option<Legal>,

    /// An array of requirement dictionaries that shall represent requirements
    /// for the document
    requirements: Option<Vec<Requirement>>,

    /// A collection dictionary that a conforming reader shall use to enhance
    /// the presentation of file attachments stored in the PDF document.
    collection: Option<Collection>,

    /// A flag used to expedite the display of PDF documents containing XFA forms.
    /// It specifies whether the document shall be regenerated when the document
    /// is first opened
    ///
    /// Default value: false.
    needs_rendering: bool,
}

pub struct Encryption;
pub struct InformationDictionary;
pub struct Extensions;
pub struct Pages;
pub struct NumberTree;
pub struct NameDictionary;
pub struct NamedDestinations;
pub struct ViewerPreferences;
pub struct DocumentOutline;
pub struct ThreadDictionary;
pub struct OpenAction;
pub struct AdditionalActions;
pub struct UriDict;
pub struct AcroForm;
pub struct MetadataStream;
pub struct StructTreeRoot;
pub struct MarkInfo;
pub struct WebCapture;
pub struct OutputIntent;
pub struct PagePiece;
pub struct OptionalContentProperties;
pub struct Permissions;
pub struct Legal;
pub struct Requirement;
pub struct Collection;

/// Specifies the page layout when the document is opened
enum PageLayout {
    /// Display one page at a time
    SinglePage,

    /// Display the pages in one column
    OneColumn,

    /// Display the pages in two columns,
    /// with odd-numbered pages on the left
    TwoColumnLeft,

    /// Display the pages in two columns,
    /// with odd-numbered pages on the right
    TwoColumnRight,

    /// Display the pages two at a time,
    /// with odd-numbered pages on the left
    TwoPageLeft,

    /// Display the pages two at a time,
    /// with odd-numbered pages on the right
    TwoPageRight,
}

impl PageLayout {
    pub fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "SinglePage" => Self::SinglePage,
            "OneColumn" => Self::OneColumn,
            "TwoColumnLeft" => Self::TwoColumnLeft,
            "TwoColumnRight" => Self::TwoColumnRight,
            "TwoPageLeft" => Self::TwoPageLeft,
            "TwoPageRight" => Self::TwoPageRight,
            _ => return Err(ParseError::Todo),
        })
    }
}

impl Default for PageLayout {
    fn default() -> Self {
        Self::SinglePage
    }
}

/// A name object specifying how the document shall be
/// displayed when opened
enum PageMode {
    /// Neither document outline nor thumbnail
    /// images visible
    UseNone,

    /// Document outline visible
    UseOutlines,

    /// Thumbnail images visible
    UseThumbs,

    /// Full-screen mode, with no menu bar, window
    /// controls, or any other window visible
    FullScreen,

    /// Optional content group panel visible
    UseOc,

    /// Attachments panel visible
    UseAttachments,
}

impl PageMode {
    pub fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "UseNone" => Self::UseNone,
            "UseOutlines" => Self::UseOutlines,
            "UseThumbs" => Self::UseThumbs,
            "FullScreen" => Self::FullScreen,
            "UseOc" => Self::UseOc,
            "UseAttachments" => Self::UseAttachments,
            _ => return Err(ParseError::Todo),
        })
    }
}

impl Default for PageMode {
    fn default() -> Self {
        Self::UseNone
    }
}

pub struct Trailer {
    /// The total number of entries in the
    /// file's cross-reference table, as
    /// defined by the combination of the
    /// original section and all update sections.
    ///
    /// Equivalently, this value shall be 1 greater
    /// than the highest object number defined in the
    /// file. Any object in a cross-reference section
    /// whose number is greater than this value shall
    /// be ignored and defined to be missing by a
    /// conforming reader.
    size: usize,

    /// The byte offset in the decoded stream from the
    /// beginning of the file to the beginning of the
    /// previous cross-reference section
    ///
    /// Present only if the file has more than one
    /// cross-reference section.
    prev: Option<usize>,

    root: DocumentCatalog,

    encryption: Option<Encryption>,
    id: Option<[String; 2]>,
    info: Option<InformationDictionary>,
}
