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
pub struct Resources {
    /// A dictionary that maps resource names to
    /// graphics state parameter dictionaries
    ext_g_state: Option<Dictionary>,
    // ext_g_state: Option<HashMap<String, GraphicsStateParameters>>,
    /// A dictionary that maps each resource name to
    /// either the name of a device-dependent color
    /// space or an array describing a color space
    color_space: Option<Dictionary>,
    // color_space: Option<HashMap<String, ColorSpace>>,
    pattern: Option<Dictionary>,
    shading: Option<Dictionary>,
    xobject: Option<Dictionary>,
    font: Option<Dictionary>,
    proc_set: Option<Vec<ProcedureSet>>,
    properties: Option<Dictionary>,
    // pattern: Option<HashMap<String, Pattern>>,
    // shading: Option<HashMap<String, Shading>>,
    // xobject: Option<HashMap<String, XObject>>,
    // font: Option<HashMap<String, Font>>,
    // properties: Option<HashMap<String, PropertyList>>,
}

impl Resources {
    pub(crate) fn from_dict(mut dict: Dictionary) -> PdfResult<Self> {
        let ext_g_state = dict.get_dict("ExtGState")?;
        let color_space = dict.get_dict("ColorSpace")?;
        let pattern = dict.get_dict("Pattern")?;
        let shading = dict.get_dict("Shading")?;
        let xobject = dict.get_dict("XObject")?;
        let font = dict.get_dict("Font")?;
        let proc_set = dict
            .get_arr("ProcSet")?
            .map(|proc| {
                proc.into_iter()
                    .map(|proc| ProcedureSet::from_str(&proc.assert_name()?))
                    .collect::<PdfResult<Vec<ProcedureSet>>>()
            })
            .transpose()?;
        let properties = dict.get_dict("Properties")?;

        Ok(Resources {
            ext_g_state,
            color_space,
            pattern,
            shading,
            xobject,
            font,
            proc_set,
            properties,
        })
    }
}

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
#[derive(Debug)]
pub struct GraphicsStateParameters;
#[derive(Debug)]
pub struct Pattern;
#[derive(Debug)]
pub struct Shading;
#[derive(Debug)]
pub struct XObject;
#[derive(Debug)]
pub struct Font;
#[derive(Debug)]
pub struct PropertyList;

#[derive(Debug)]
pub enum ProcedureSet {
    Pdf,
    Text,
    ImageB,
    ImageC,
    ImageI,
}

impl ProcedureSet {
    pub(crate) fn from_str(s: &str) -> PdfResult<Self> {
        Ok(match s {
            "PDF" => Self::Pdf,
            "Text" => Self::Text,
            "ImageB" => Self::ImageB,
            "ImageC" => Self::ImageC,
            "ImageI" => Self::ImageI,
            _ => return Err(ParseError::Todo),
        })
    }
}

#[derive(Debug)]
#[allow(unused)]
pub enum ColorSpace {
    // Device
    DeviceGray(f32),
    DeviceRGB {
        red: f32,
        green: f32,
        blue: f32,
    },
    DeviceCMYK {
        cyan: f32,
        magenta: f32,
        yellow: f32,
        key: f32,
    },

    // CIE-based
    CalGray,
    CalRGB,
    Lab,
    ICCBased,

    // Special
    Indexed,
    Pattern,
    Separation,
    DeviceN,
}

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
