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
    destination::Destination, objects::Name, optional_content::OptionalContentProperties,
    stream::Stream, structure::StructTreeRoot, viewer_preferences::ViewerPreferences, Dictionary,
    FromObj, Lexer, Object, ParseError, PdfResult, Reference, Resolve,
};

pub use crate::color::{ColorSpace, ColorSpaceName};

/// See module level documentation
#[derive(Debug)]
pub struct DocumentCatalog<'a> {
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
    page_labels: Option<NumberTree<'a>>,

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
    open_action: Option<OpenAction<'a>>,

    /// An additional-actions dictionary defining the actions
    /// that shall be taken in response to various trigger
    /// events affecting the document as a whole
    aa: Option<AdditionalActions>,

    /// A URI dictionary containing document-level information
    /// for URI actions
    uri: Option<UriDict>,

    acro_form: Option<AcroForm>,

    metadata: Option<Reference>,

    struct_tree_root: Option<StructTreeRoot<'a>>,

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

    piece_info: Option<PagePiece<'a>>,

    /// The document's optional content properties dictionary
    ///
    /// Required if a document contains optional content
    oc_properties: Option<OptionalContentProperties<'a>>,

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

    last_modified: Option<Date>,
}

impl<'a> DocumentCatalog<'a> {
    const TYPE: &'static str = "Catalog";

    pub(crate) fn from_dict(mut dict: Dictionary<'a>, lexer: &mut Lexer<'a>) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, lexer, true)?;

        let version = dict.get_name("Version", lexer)?;
        let extensions = None;
        let pages = dict.expect_reference("Pages")?;
        let page_labels = None;
        let names = None;
        let dests = dict.get_reference("Dests")?;
        let viewer_preferences = dict.get::<ViewerPreferences>("ViewerPreferences", lexer)?;

        let page_layout = dict
            .get_name("PageLayout", lexer)?
            .as_deref()
            .map(PageLayout::from_str)
            .unwrap_or_else(|| Ok(PageLayout::default()))?;
        let page_mode = dict
            .get_name("PageMode", lexer)?
            .as_deref()
            .map(PageMode::from_str)
            .unwrap_or_else(|| Ok(PageMode::default()))?;

        let outlines = dict.get_reference("Outlines")?;
        let threads = dict.get_reference("Threads")?;
        let open_action = dict.get::<OpenAction>("OpenAction", lexer)?;
        let aa = None;
        let uri = None;
        let acro_form = None;
        let metadata = dict.get_reference("Metadata")?;
        let struct_tree_root = dict
            .get_dict("StructTreeRoot", lexer)?
            .map(|dict| StructTreeRoot::from_dict(dict, lexer))
            .transpose()?;
        let mark_info = dict.get::<MarkInformationDictionary>("MarkInfo", lexer)?;
        let lang = dict.get_string("Lang", lexer)?;
        let spider_info = None;
        let output_intents = None;
        let piece_info = dict.get::<PagePiece>("PieceInfo", lexer)?;
        let oc_properties = dict.get::<OptionalContentProperties>("OCProperties", lexer)?;
        let perms = None;
        let legal = None;
        let requirements = None;
        let collection = None;
        let needs_rendering = dict.get_bool("NeedsRendering", lexer)?.unwrap_or(false);
        let last_modified = dict.get::<Date>("LastModified", lexer)?;

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
            last_modified,
        })
    }
}

#[derive(Debug)]
pub struct Encryption;

#[derive(Debug, FromObj)]
pub struct InformationDictionary<'a> {
    #[field("Title")]
    title: Option<String>,
    #[field("Author")]
    author: Option<String>,
    #[field("Subject")]
    subject: Option<String>,
    #[field("Keywords")]
    keywords: Option<String>,

    /// If the document was converted to PDF from
    /// another format, the name of the conforming
    /// product that created the original document
    /// from which it was converted
    #[field("Creator")]
    creator: Option<String>,

    /// If the document was converted to PDF from
    /// another format, the name of the conforming
    /// product that converted it to PDF
    #[field("Producer")]
    producer: Option<String>,

    #[field("CreationDate")]
    creation_date: Option<Date>,
    #[field("ModDate")]
    mod_date: Option<Date>,
    #[field("Trapped", default = Trapped::default())]
    trapped: Trapped,

    #[field("")]
    other: Dictionary<'a>,
}

/// A name object indicating whether the document
/// has been modified to include trapping information
#[pdf_enum]
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
            // found: arr.to_vec(),
        });
    }

    Ok(())
}

#[derive(Debug)]
pub enum OpenAction<'a> {
    Destination(Destination),
    Actions(Actions<'a>),
}

impl<'a> FromObj<'a> for OpenAction<'a> {
    fn from_obj(obj: Object<'a>, lexer: &mut dyn Resolve<'a>) -> PdfResult<Self> {
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
#[derive(Debug, Clone)]
pub struct MetadataStream<'a> {
    stream: Stream<'a>,
    subtype: MetadataStreamSubtype,
}

#[pdf_enum]
enum MetadataStreamSubtype {
    Xml = "XML",
}

impl<'a> MetadataStream<'a> {
    const TYPE: &'static str = "Metadata";

    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        dict.expect_type(Self::TYPE, resolver, true)?;

        let subtype = MetadataStreamSubtype::from_str(&dict.expect_name("Subtype", resolver)?)?;

        Ok(Self { stream, subtype })
    }
}

#[derive(Debug, FromObj)]
pub struct MarkInformationDictionary {
    /// A flag indicating whether the document conforms to Tagged PDF conventions.
    ///
    /// Default value: false.
    /// If Suspects is true, the document may not completely conform to Tagged PDF conventions.
    #[field("Marked", default = false)]
    marked: bool,

    /// A flag indicating the presence of structure elements that contain user properties attributes
    ///
    /// Default value: false
    #[field("UserProperties", default = false)]
    user_properties: bool,

    /// A flag indicating the presence of tag suspects
    ///
    /// Default value: false.
    #[field("Suspects", default = false)]
    suspects: bool,
}

#[derive(Debug)]
pub struct WebCapture;
#[derive(Debug)]
pub struct OutputIntent;

#[derive(Debug, Clone)]
pub struct PagePiece<'a>(Dictionary<'a>);

impl<'a> FromObj<'a> for PagePiece<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = resolver.assert_dict(obj)?;
        Ok(Self(dict))
    }
}

#[derive(Debug)]
pub struct Permissions;
#[derive(Debug)]
pub struct Legal;
#[derive(Debug)]
pub struct Requirement;
#[derive(Debug, Clone, PartialEq)]
pub struct Collection;
#[derive(Debug)]
pub struct BoxColorInfo;

#[derive(Debug, Clone, FromObj)]
pub struct GroupAttributes<'a> {
    /// The group subtype, which identifies the type of group whose attributes
    /// this dictionary describes. This is always "Transparency"
    #[field("S")]
    subtype: Name,

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
    #[field("CS")]
    cs: Option<Object<'a>>,

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
    #[field("I", default = false)]
    is_isolated: bool,

    /// A flag specifying whether the transparency group is a knockout group.
    ///
    /// If this flag is false, later objects within the group shall be composited with earlier
    /// ones with which they overlap; if true, they shall be composited with the group's initial
    /// backdrop and shall overwrite ("knock out") any earlier overlapping objects.
    ///
    /// Default value: false.
    #[field("K", default = false)]
    is_knockout: bool,
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
pub struct PropertyList;

#[pdf_enum]
/// Specifies the page layout when the document is opened
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

impl Default for PageLayout {
    fn default() -> Self {
        Self::SinglePage
    }
}

#[pdf_enum]
/// A name object specifying how the document shall be
/// displayed when opened
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

impl Default for PageMode {
    fn default() -> Self {
        Self::UseNone
    }
}
