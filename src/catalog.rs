/*!
The catalog contains references to other objects defining
the document's contents, outline, article threads, named
destinations, and other attributes.

In addition, it contains information about how the
document shall be displayed on the screen, such as
whether its outline and thumbnail page images shall
be displayed automatically and whether some location
other than the first page shall be shown when the
document is opened.
*/
use std::{cell::RefCell, fmt, rc::Rc};

use crate::{Dictionary, Object, ParseError, PdfResult, Reference};

/// See module level documentation
#[derive(Debug)]
pub struct DocumentCatalog {
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
    version: Option<String>,

    extensions: Option<Extensions>,

    pub pages: Reference,

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

    /// A dictionary of names and corresponding destinations
    dests: Option<Reference>,

    // A viewer preferences dictionary specifying the way
    /// the document shall be displayed on the screen. If
    /// this entry is absent, conforming readers shall use
    /// their own current user preference settings.
    viewer_preferences: Option<ViewerPreferences>,

    page_layout: PageLayout,

    page_mode: PageMode,

    outlines: Option<Reference>,

    threads: Option<Reference>,

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

    metadata: Option<Reference>,

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

impl DocumentCatalog {
    pub(crate) fn from_dict(mut dict: Dictionary) -> PdfResult<Self> {
        if dict.expect_name("Type")? != "Catalog" {
            todo!()
        }

        let version = dict.get_name("Version")?;
        let extensions = None;
        let pages = dict.expect_reference("Pages")?;
        let page_labels = None;
        let names = None;
        let dests = None;
        let viewer_preferences = None;

        let page_layout = dict
            .get_name("PageLayout")?
            .as_deref()
            .map(PageLayout::from_str)
            .unwrap_or(Ok(PageLayout::default()))?;
        let page_mode = dict
            .get_name("PageMode")?
            .as_deref()
            .map(PageMode::from_str)
            .unwrap_or(Ok(PageMode::default()))?;

        let outlines = dict.get_reference("Outlines")?;
        let threads = dict.get_reference("Threads")?;
        let open_action = None;
        let aa = None;
        let uri = None;
        let acro_form = None;
        let metadata = dict.get_reference("Metadata")?;
        let struct_tree_root = None;
        let mark_info = None;
        let lang = dict.get_string("Lang")?;
        let spider_info = None;
        let output_intents = None;
        let piece_info = None;
        let oc_properties = None;
        let perms = None;
        let legal = None;
        let requirements = None;
        let collection = None;
        let needs_rendering = dict.get_bool("NeedsRendering")?.unwrap_or(false);

        if !dict.is_empty() {
            todo!("dict not empty: {:#?}", dict);
        }

        Ok(DocumentCatalog {
            version,
            extensions,
            pages,
            page_labels,
            names,
            dests,
            viewer_preferences,
            page_layout,
            page_mode,
            outlines,
            threads,
            open_action,
            aa,
            uri,
            acro_form,
            metadata,
            struct_tree_root,
            mark_info,
            lang,
            spider_info,
            output_intents,
            piece_info,
            oc_properties,
            perms,
            legal,
            requirements,
            collection,
            needs_rendering,
        })
    }
}

#[derive(Debug)]
pub struct Encryption;
#[derive(Debug)]
pub struct InformationDictionary;
#[derive(Debug)]
pub struct Extensions;

pub struct PageTree {
    pub kids: Vec<PageNode>,
    // kids: Vec<Reference>,
    pub count: usize,
}

#[derive(Clone)]
pub enum PageNode {
    Root(Rc<RefCell<PageTree>>),
    Node(Rc<RefCell<PageTreeNode>>),
    Leaf(Rc<PageObject>),
}

impl fmt::Debug for PageNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Root(r) => f
                .debug_struct("PageNode::Root")
                .field("kids", &r.borrow().kids)
                .field("count", &r.borrow().count)
                .finish(),
            Self::Node(r) => f
                .debug_struct("PageNode::Node")
                // .field("parent", &r.borrow().parent)
                .field("kids", &r.borrow().kids)
                .field("count", &r.borrow().count)
                .finish(),
            Self::Leaf(r) => f
                .debug_struct("PageNode::Leaf")
                .field("resources", &r.resources)
                .finish(),
        }
    }
}

// "Pages"
pub struct PageTreeNode {
    /// The page tree node that is the immediate parent of this one.
    ///
    /// Required except in root node; prohibited in the root node
    pub parent: PageNode,

    /// An array of indirect references to the immediate children
    /// of this node. The children shall only be page objects or
    /// other page tree nodes.
    pub kids: Vec<PageNode>,

    /// The number of leaf nodes (page objects) that are descendants
    /// of this node within the page tree
    pub count: usize,
}

// "Page"
pub struct PageObject {
    pub parent: PageNode,
    /// The date and time when the page's contents were most recently
    /// modified. If a page-piece dictionary ([`PieceInfo`](crate::PieceInfo))
    /// is present, the modification date shall be used to ascertain
    /// which of the application data dictionaries that it contains
    /// correspond to the current content of the page.
    pub last_modified: Option<Date>,

    /// A dictionary containing any resources required by the page. If the
    /// page requires no resources, the value of this entry shall be an
    /// empty dictionary. Omitting the entry entirely indicates that the
    /// resources shall be inherited from an ancestor node in the page tree.
    pub resources: Resources,

    /// A rectangle, expressed in default user space units, that shall define
    /// the boundaries of the physical medium on which the page shall be displayed
    /// or printed.
    pub media_box: Rectangle,

    /// A rectangle, expressed in default user space units, that shall
    /// define the visible region of default user space. When the page
    /// is displayed or printed, its contents shall be clipped (cropped)
    /// to this rectangle and then shall be imposed on the output medium
    /// in some implementation-defined manner.
    ///
    /// Default value: the value of `media_box`.
    pub crop_box: Option<Rectangle>,

    /// A rectangle, expressed in default user space units, that shall
    /// define the region to which the contents of the page shall be
    /// clipped when output in a production environment.
    ///
    /// Default value: the value of `crop_box`.
    pub bleed_box: Option<Rectangle>,

    /// A rectangle, expressed in default user space units, that shall
    /// define the intended dimensions of the finished page after trimming.
    ///
    /// Default value: the value of `crop_box`.
    pub trim_box: Option<Rectangle>,

    /// A rectangle, expressed in default user space units, that shall
    /// define the extent of the page's meaningful content (including
    /// potential white space) as intended by the page's creator.
    ///
    /// Default value: the value of `crop_box`.
    pub art_box: Option<Rectangle>,

    /// A box colour information dictionary that shall specify the
    /// colours and other visual characteristics that should be used
    /// in displaying guidelines on the screen for the various page
    /// boundaries.
    ///
    /// If this entry is absent, the application shall use its own
    /// current default settings.
    pub box_color_info: Option<BoxColorInfo>,

    /// A content stream that shall describe the contents of this page.
    ///
    /// If this entry is absent, the page shall be empty.
    ///
    /// The value shall be either a single stream or an array of streams.
    /// If the value is an array, the effect shall be as if all of the
    /// streams in the array were concatenated, in order, to form a single
    /// stream. Conforming writers can create image objects and other
    /// resources as they occur, even though they interrupt the content stream.
    ///
    /// The division between streams may occur only at the boundaries between
    /// lexical tokens but shall be unrelated to the page's logical content
    /// or organization. Applications that consume or produce PDF files need
    /// not preserve the existing structure of the Contents array.
    ///
    /// Conforming writers shall not create a Contents array containing no elements.
    pub contents: Option<ContentStream>,

    /// The number of degrees by which the page shall be rotated clockwise
    /// when displayed or printed. The value shall be a multiple of 90.
    ///
    /// Default value: 0.
    pub rotate: i32,

    /// A group attributes dictionary that shall specify the attributes of
    /// the page's page group for use in the transparent imaging model
    pub group: Option<GroupAttributes>,

    /// A stream object that shall define the page's thumbnail image
    pub thumb: Option<Vec<u8>>,

    /// An array that shall contain indirect references to all article beads
    /// appearing on the page. The beads shall be listed in the array in
    /// natural reading order.
    pub b: Option<Vec<Reference>>,

    /// The page's display duration (also called its advance timing): the
    /// maximum length of time, in seconds, that the page shall be displayed
    /// during presentations before the viewer application shall automatically
    /// advance to the next page.
    ///
    /// By default, the viewer shall not advance automatically.
    // TODO: type=number?
    pub dur: Option<f32>,

    /// A transition dictionary describing the transition effect that shall
    /// be used when displaying the page during presentations
    pub trans: Option<Transitions>,

    /// An array of annotation dictionaries that shall contain indirect
    /// references to all annotations associated with the page
    pub annots: Option<Vec<Annotation>>,

    /// An additional-actions dictionary that shall define actions to
    /// be performed when the page is opened or closed
    pub aa: Option<AdditionalActions>,

    pub metadata: Option<MetadataStream>,
    pub piece_info: Option<PagePiece>,

    /// The integer key of the page's entry in the structural parent tree
    pub struct_parents: Option<i32>,

    /// The digital identifier of the page's parent Web Capture content set
    pub id: Option<String>,

    /// The page's preferred zoom (magnification) factor: the factor by
    /// which it shall be scaled to achieve the natural display magnification
    pub pz: Option<f32>,

    /// A separation dictionary that shall contain information needed to
    /// generate colour separations for the page
    pub separation_info: Option<SeparationInfo>,

    /// A name specifying the tab order that shall be used for annotations on the page.
    ///
    /// The possible values shall be
    ///   * R (row order)
    ///   * C (column order)
    ///   * S (structure order).
    pub tabs: Option<TabOrder>,

    /// The name of the originating page object
    pub template_instantiated: Option<String>,

    /// A navigation node dictionary that shall represent the first
    /// node on the page
    pub pres_steps: Option<NavigationNode>,

    /// A positive number that shall give the size of default user space units,
    /// in multiples of 1/72 inch. The range of supported values shall be
    /// mplementation-dependent.
    ///
    /// Default value: 1.0 (user space unit is 1/72 inch).
    pub user_unit: Option<f32>,

    /// An array of viewport dictionaries that shall specify rectangular
    /// regions of the page.
    pub vp: Option<Viewport>,
}

#[derive(Debug)]
pub enum TabOrder {
    Row,
    Column,
    Structure,
}

#[derive(Debug)]
pub struct NumberTree;
#[derive(Debug)]
pub struct NameDictionary;
#[derive(Debug)]
pub struct NamedDestinations;
#[derive(Debug)]
pub struct ViewerPreferences;
#[derive(Debug)]
pub struct DocumentOutline;
#[derive(Debug)]
pub struct ThreadDictionary;
#[derive(Debug)]
pub struct OpenAction;
#[derive(Debug)]
pub struct AdditionalActions;
#[derive(Debug)]
pub struct UriDict;
#[derive(Debug)]
pub struct AcroForm;
#[derive(Debug)]
pub struct MetadataStream;
#[derive(Debug)]
pub struct StructTreeRoot;
#[derive(Debug)]
pub struct MarkInfo;
#[derive(Debug)]
pub struct WebCapture;
#[derive(Debug)]
pub struct OutputIntent;
#[derive(Debug)]
pub struct PagePiece;
#[derive(Debug)]
pub struct OptionalContentProperties;
#[derive(Debug)]
pub struct Permissions;
#[derive(Debug)]
pub struct Legal;
#[derive(Debug)]
pub struct Requirement;
#[derive(Debug)]
pub struct Collection;
#[derive(Debug)]
pub struct Resources;
#[derive(Debug)]
pub struct Date;
#[derive(Debug)]
pub struct Rectangle;
#[derive(Debug)]
pub struct BoxColorInfo;
#[derive(Debug)]
pub struct ContentStream;
#[derive(Debug)]
pub struct GroupAttributes;
#[derive(Debug)]
pub struct Transitions;
#[derive(Debug)]
pub struct Annotation;
#[derive(Debug)]
pub struct SeparationInfo;
#[derive(Debug)]
pub struct NavigationNode;
#[derive(Debug)]
pub struct Viewport;

/// Specifies the page layout when the document is opened
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
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
    pub size: usize,

    /// The byte offset in the decoded stream from the
    /// beginning of the file to the beginning of the
    /// previous cross-reference section
    ///
    /// Present only if the file has more than one
    /// cross-reference section.
    pub prev: Option<usize>,

    pub root: Reference,

    pub encryption: Option<Encryption>,
    id: Option<Vec<Object>>,
    // pub id: Option<[String; 2]>,
    pub info: Option<Reference>,
}

impl Trailer {
    pub(crate) fn from_dict(mut dict: Dictionary) -> PdfResult<Self> {
        let size = dict.expect_integer("Size")? as usize;
        let prev = dict.get_integer("Prev")?.map(|i| i as usize);
        let root = dict.expect_reference("Root")?;
        // TODO: encryption dicts
        let encryption = None;
        let id = dict.get_arr("ID")?;
        let info = dict.get_reference("Info")?;

        if !dict.is_empty() {
            todo!("dict not empty: {:#?}", dict);
        }

        Ok(Trailer {
            size,
            prev,
            root,
            encryption,
            info,
            id,
        })
    }
}
