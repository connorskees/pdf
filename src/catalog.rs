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

use crate::{
    actions::Actions, assert_empty, data_structures::NumberTree, date::Date,
    destination::Destination, pdf_enum, stream::Stream, structure::StructTreeRoot,
    viewer_preferences::ViewerPreferences, Dictionary, Lexer, Object, ParseError, PdfResult,
    Reference, Resolve,
};

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

    /// An extensions dictionary containing developer prefix
    /// identification and version numbers for developer extensions
    /// that occur in this document
    extensions: Option<Extensions>,

    /// The page tree node that shall be the root of the document's
    /// page tree
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

    /// The document's name dictionary
    names: Option<NameDictionary>,

    /// A dictionary of names and corresponding destinations
    dests: Option<Reference>,

    /// A viewer preferences dictionary specifying the way
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
    ///
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
    mark_info: Option<MarkInformationDictionary>,

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
    const TYPE: &'static str = "Catalog";

    pub(crate) fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, lexer, true)?;

        let version = dict.get_name("Version", lexer)?;
        let extensions = None;
        let pages = dict.expect_reference("Pages")?;
        let page_labels = None;
        let names = None;
        let dests = dict.get_reference("Dests")?;
        let viewer_preferences = dict
            .get_dict("ViewerPreferences", lexer)?
            .map(|dict| ViewerPreferences::from_dict(dict, lexer))
            .transpose()?;

        let page_layout = dict
            .get_name("PageLayout", lexer)?
            .as_deref()
            .map(PageLayout::from_str)
            .unwrap_or(Ok(PageLayout::default()))?;
        let page_mode = dict
            .get_name("PageMode", lexer)?
            .as_deref()
            .map(PageMode::from_str)
            .unwrap_or(Ok(PageMode::default()))?;

        let outlines = dict.get_reference("Outlines")?;
        let threads = dict.get_reference("Threads")?;
        let open_action = dict
            .get_object("OpenAction", lexer)?
            .map(|obj| OpenAction::from_obj(obj, lexer))
            .transpose()?;
        let aa = None;
        let uri = None;
        let acro_form = None;
        let metadata = dict.get_reference("Metadata")?;
        let struct_tree_root = dict
            .get_dict("StructTreeRoot", lexer)?
            .map(|dict| StructTreeRoot::from_dict(dict, lexer))
            .transpose()?;
        let mark_info = dict
            .get_dict("MarkInfo", lexer)?
            .map(|dict| MarkInformationDictionary::from_dict(dict, lexer))
            .transpose()?;
        let lang = dict.get_string("Lang", lexer)?;
        let spider_info = None;
        let output_intents = None;
        let piece_info = None;
        let oc_properties = None;
        let perms = None;
        let legal = None;
        let requirements = None;
        let collection = None;
        let needs_rendering = dict.get_bool("NeedsRendering", lexer)?.unwrap_or(false);

        assert_empty(dict);

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
pub struct InformationDictionary {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    keywords: Option<String>,

    /// If the document was converted to PDF from
    /// another format, the name of the conforming
    /// product that created the original document
    /// from which it was converted
    creator: Option<String>,

    /// If the document was converted to PDF from
    /// another format, the name of the conforming
    /// product that converted it to PDF
    producer: Option<String>,

    creation_date: Option<Date>,
    mod_date: Option<Date>,
    trapped: Trapped,

    other: Dictionary,
}

impl InformationDictionary {
    pub(crate) fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let title = dict.get_string("Title", lexer)?;
        let author = dict.get_string("Author", lexer)?;
        let subject = dict.get_string("Subject", lexer)?;
        let keywords = dict.get_string("Keywords", lexer)?;
        let creator = dict.get_string("Creator", lexer)?;
        let producer = dict.get_string("Producer", lexer)?;
        let creation_date = dict
            .get_string("CreationDate", lexer)?
            .as_deref()
            .map(Date::from_str)
            .transpose()?;
        let mod_date = dict
            .get_string("ModDate", lexer)?
            .as_deref()
            .map(Date::from_str)
            .transpose()?;
        let trapped = dict
            .get_name("Trapped", lexer)?
            .as_deref()
            .map(Trapped::from_str)
            .transpose()?
            .unwrap_or_default();

        Ok(InformationDictionary {
            title,
            author,
            subject,
            keywords,
            creator,
            producer,
            creation_date,
            mod_date,
            trapped,
            other: dict,
        })
    }
}

pdf_enum!(
    /// A name object indicating whether the document
    /// has been modified to include trapping information
    #[derive(Debug)]
    pub enum Trapped {
        /// The document has been fully trapped; no further
        /// trapping shall be needed. This shall be the name
        /// "True", not the boolean value true.
        True = "True",

        /// The document has not yet been trapped. This shall
        /// be the name "False", not the boolean value false
        False = "False",

        /// Either it is unknown whether the document has been
        /// trapped or it has been partly but not yet fully
        /// trapped; some additional trapping may still be needed
        Unknown = "Unknown",
    }
);

impl Default for Trapped {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct Extensions;
#[derive(Debug)]
struct Language;
#[derive(Debug)]
pub struct NameDictionary;
#[derive(Debug)]
pub struct NamedDestinations;
#[derive(Debug)]
pub struct DocumentOutline;
#[derive(Debug)]
pub struct ThreadDictionary;

pub fn assert_len(arr: &[Object], len: usize) -> PdfResult<()> {
    if arr.len() != len {
        return Err(ParseError::ArrayOfInvalidLength {
            expected: len,
            found: arr.to_vec(),
        });
    }

    Ok(())
}

#[derive(Debug)]
pub enum OpenAction {
    Destination(Destination),
    Actions(Actions),
}

impl OpenAction {
    pub fn from_obj(obj: Object, lexer: &mut Lexer) -> PdfResult<Self> {
        let obj = lexer.resolve(obj)?;
        Ok(match obj {
            Object::Dictionary(dict) => OpenAction::Actions(Actions::from_dict(dict, lexer)?),
            obj => OpenAction::Destination(Destination::from_obj(obj, lexer)?),
        })
    }
}

#[derive(Debug)]
pub struct AdditionalActions;
#[derive(Debug)]
pub struct UriDict;
#[derive(Debug)]
pub struct AcroForm;
#[derive(Debug)]
pub struct MetadataStream;

impl MetadataStream {
    pub fn from_stream(_stream: Stream, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct MarkInformationDictionary {
    /// A flag indicating whether the document conforms to Tagged PDF conventions.
    ///
    /// Default value: false.
    /// If Suspects is true, the document may not completely conform to Tagged PDF conventions.
    marked: bool,

    /// A flag indicating the presence of structure elements that contain user properties attributes
    ///
    /// Default value: false
    user_properties: bool,

    /// A flag indicating the presence of tag suspects
    ///
    /// Default value: false.
    suspects: bool,
}

impl MarkInformationDictionary {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let marked = dict.get_bool("Marked", resolver)?.unwrap_or(false);
        let user_properties = dict.get_bool("UserProperties", resolver)?.unwrap_or(false);
        let suspects = dict.get_bool("Suspects", resolver)?.unwrap_or(false);

        assert_empty(dict);

        Ok(Self {
            marked,
            user_properties,
            suspects,
        })
    }
}

#[derive(Debug)]
pub struct WebCapture;
#[derive(Debug)]
pub struct OutputIntent;
#[derive(Debug)]
pub struct PagePiece;

impl PagePiece {
    pub fn from_dict(_dict: Dictionary, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct OptionalContent;
impl OptionalContent {
    pub fn from_dict(_dict: Dictionary, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
pub struct OptionalContentProperties;

impl OptionalContentProperties {
    pub fn from_dict(_dict: Dictionary, _resolver: &mut dyn Resolve) -> PdfResult<Self> {
        todo!()
    }
}
#[derive(Debug)]
pub struct Permissions;
#[derive(Debug)]
pub struct Legal;
#[derive(Debug)]
pub struct Requirement;
#[derive(Debug, Clone)]
pub struct Collection;
#[derive(Debug)]
pub struct BoxColorInfo;
#[derive(Debug)]
pub struct ContentStream;

#[derive(Debug)]
pub struct GroupAttributes {
    /// The group colour space, which is used for the following purposes:
    ///
    ///  * As the colour space into which colours shall be converted when painted into the
    ///    group
    ///  * As the blending colour space in which objects shall be composited within the group
    ///  * As the colour space of the group as a whole when it in turn is painted as an object
    ///    onto its backdrop
    ///
    /// The group colour space shall be any device or CIE-based colour space that treats its
    /// components as independent additive or subtractive values in the range 0.0 to 1.0,
    /// subject to the restrictions described in Blending Colour Space. These restrictions
    /// exclude Lab and lightnesschromaticity ICCBased colour spaces, as well as the special
    /// colour spaces Pattern, Indexed, Separation, and DeviceN. Device colour spaces shall be
    /// subject to remapping according to the DefaultGray, DefaultRGB, and DefaultCMYK entries
    /// in the ColorSpace subdictionary of the current resource dictionary.
    ///
    /// Ordinarily, the CS entry may be present only for isolated transparency groups (those
    /// for which I is true), and even then it is optional. However, this entry shall be present
    /// in the group attributes dictionary for any transparency group XObject that has no parent
    /// group or page from which to inherit -- in particular, one that is the value of the G entry
    /// in a soft-mask dictionary of subtype Luminosity.
    ///
    /// Additionally, the CS entry may be present in the group attributes dictionary associated
    /// with a page object, even if I is false or absent. In the normal case in which the page
    /// is imposed directly on the output medium, the page group is effectively isolated regardless
    /// of the I value, and the specified CS value shall therefore be honoured. But if the page
    /// is in turn used as an element of some other page and if the group is nonisolated, CS shall
    /// be ignored and the colour space shall be inherited from the actual backdrop with which the
    /// page is composited.
    ///
    /// Default value: the colour space of the parent group or page into which this transparency
    /// group is painted. (The parent's colour space in turn may be either explicitly specified or
    /// inherited.)
    ///
    /// For a transparency group XObject used as an annotation appearance, the default colour space
    /// shall be inherited from the page on which the annotation appears
    // todo: type
    cs: Option<Object>,

    /// A flag specifying whether the transparency group is isolated.
    ///
    /// If this flag is true, objects within the group shall be composited against a fully
    /// transparent initial backdrop; if false, they shall be composited against the group's
    /// backdrop.
    ///
    /// Default value: false.
    ///
    /// In the group attributes dictionary for a page, the interpretation of this entry shall
    /// be slightly altered. In the normal case in which the page is imposed directly on the
    /// output medium, the page group is effectively isolated and the specified I value shall
    /// be ignored. But if the page is in turn used as an element of some other page, it shall
    /// be treated as if it were a transparency group XObject; the I value shall be interpreted
    /// in the normal way to determine whether the page group is isolated.
    is_isolated: bool,

    /// A flag specifying whether the transparency group is a knockout group.
    ///
    /// If this flag is false, later objects within the group shall be composited with earlier
    /// ones with which they overlap; if true, they shall be composited with the group's initial
    /// backdrop and shall overwrite ("knock out") any earlier overlapping objects.
    ///
    /// Default value: false.
    is_knockout: bool,
}

impl GroupAttributes {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let s = dict.expect_name("S", resolver)?;
        if s != "Transparency" {
            todo!()
        }

        let cs = dict.get_object("CS", resolver)?;
        let is_isolated = dict.get_bool("I", resolver)?.unwrap_or(false);
        let is_knockout = dict.get_bool("K", resolver)?.unwrap_or(false);

        Ok(GroupAttributes {
            cs,
            is_isolated,
            is_knockout,
        })
    }
}

#[derive(Debug)]
pub struct Transitions;
#[derive(Debug)]
pub struct SeparationInfo;
#[derive(Debug)]
pub struct NavigationNode;
#[derive(Debug)]
pub struct Viewport;
#[derive(Debug)]
pub struct Shading;
#[derive(Debug)]
pub struct Font;
#[derive(Debug)]
pub struct PropertyList;

#[derive(Debug)]
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

pdf_enum!(
    /// Specifies the page layout when the document is opened
    #[derive(Debug)]
    enum PageLayout {
        /// Display one page at a time
        SinglePage = "SinglePage",

        /// Display the pages in one column
        OneColumn = "OneColumn",

        /// Display the pages in two columns,
        /// with odd-numbered pages on the left
        TwoColumnLeft = "TwoColumnLeft",

        /// Display the pages in two columns,
        /// with odd-numbered pages on the right
        TwoColumnRight = "TwoColumnRight",

        /// Display the pages two at a time,
        /// with odd-numbered pages on the left
        TwoPageLeft = "TwoPageLeft",

        /// Display the pages two at a time,
        /// with odd-numbered pages on the right
        TwoPageRight = "TwoPageRight",
    }
);

impl Default for PageLayout {
    fn default() -> Self {
        Self::SinglePage
    }
}

pdf_enum!(
    /// A name object specifying how the document shall be
    /// displayed when opened
    #[derive(Debug)]
    pub enum PageMode {
        /// Neither document outline nor thumbnail
        /// images visible
        UseNone = "UseNone",

        /// Document outline visible
        UseOutlines = "UseOutlines",

        /// Thumbnail images visible
        UseThumbs = "UseThumbs",

        /// Full-screen mode, with no menu bar, window
        /// controls, or any other window visible
        FullScreen = "FullScreen",

        /// Optional content group panel visible
        UseOc = "UseOc",

        /// Attachments panel visible
        UseAttachments = "UseAttachments",
    }
);

impl Default for PageMode {
    fn default() -> Self {
        Self::UseNone
    }
}
