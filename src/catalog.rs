/*!
The catalog contains references to other objects defining the document's contents,
outline, article threads, named destinations, and other attributes.

In addition, it contains information about how the document shall be displayed
on the screen, such as whether its outline and thumbnail page images shall be
displayed automatically and whether some location other than the first page shall
be shown when the document is opened.
*/

use crate::{
    actions::Actions,
    data_structures::{NameTree, NumberTree},
    date::Date,
    destination::Destination,
    objects::{Name, TypedReference},
    optional_content::OptionalContentProperties,
    stream::Stream,
    structure::StructTreeRoot,
    viewer_preferences::ViewerPreferences,
    Dictionary, FromObj, Object, ParseError, PdfResult, Reference, Resolve,
};

pub use crate::color::{ColorSpace, ColorSpaceName};

/// See module level documentation
#[derive(Debug, FromObj)]
#[obj_type("Catalog")]
pub struct DocumentCatalog<'a> {
    /// The version of the PDF specification to which the document conforms(for
    /// example, 1.4) if later than the version specified in the file's header.
    ///
    /// If the header specifies a later version, or if this entry is absent, the
    /// document shall conform to the version specified in the header. This entry
    /// enables a conforming writer to update the version using an incremental
    /// update.
    ///
    /// The value of this entry shall be a name object, not a number, and
    /// therefore shall be preceded by a SOLIDUS (2Fh) character (/) when written
    /// in the PDF file (for example, /1.4).
    #[field("Version")]
    version: Option<String>,

    /// An extensions dictionary containing developer prefix identification and
    /// version numbers for developer extensions that occur in this document
    #[field("Extensions")]
    extensions: Option<Extensions>,

    /// The page tree node that shall be the root of the document's
    /// page tree
    #[field("Pages")]
    pub pages: Reference,

    /// A number tree defining the page labelling for the document. The keys in
    /// this tree shall be page indices; the corresponding values shall be page
    /// label dictionaries. Each page index shall denote the first page in a
    /// labelling range to which the specified page label dictionary applies. The
    /// tree shall include a value for page index 0.
    #[field("PageLabels")]
    page_labels: Option<TypedReference<'a, NumberTree<'a>>>,

    /// The document's name dictionary
    #[field("Names")]
    names: Option<TypedReference<'a, NameDictionary<'a>>>,

    /// A dictionary of names and corresponding destinations
    #[field("Dests")]
    dests: Option<Reference>,

    /// A viewer preferences dictionary specifying the way the document shall
    /// be displayed on the screen. If this entry is absent, conforming readers
    /// shall use their own current user preference settings.
    #[field("ViewerPreferences")]
    viewer_preferences: Option<TypedReference<'a, ViewerPreferences>>,

    /// A name object specifying the page layout shall be used when the document
    /// is opened
    #[field("PageLayout", default = PageLayout::default())]
    page_layout: PageLayout,

    /// A name object specifying how the document shall be displayed when opened
    #[field("PageMode", default = PageMode::default())]
    page_mode: PageMode,

    /// The outline dictionary that shall be the root of the document’s outline
    /// hierarchy
    ///
    /// Shall be an indirect reference
    #[field("Outlines")]
    outlines: Option<TypedReference<'a, DocumentOutline>>,

    /// An array of thread dictionaries that shall represent the document’s
    /// article threads
    ///
    /// Shall be an indirect reference
    #[field("Threads")]
    threads: Option<TypedReference<'a, ThreadDictionary>>,

    /// A value specifying a destination that shall be displayed or an action
    /// that shall be performed when the document is opened. The value shall be
    /// either an array defining a destination or an action dictionary representing
    /// an action.
    ///
    /// If this entry is absent, the document shall be opened to the top of the
    /// first page at the default magnification factor.
    #[field("OpenAction")]
    open_action: Option<OpenAction<'a>>,

    /// An additional-actions dictionary defining the actions that shall be taken
    /// in response to various trigger events affecting the document as a whole
    #[field("AA")]
    aa: Option<AdditionalActions>,

    /// A URI dictionary containing document-level information for URI actions
    #[field("URI")]
    uri: Option<UriDict>,

    /// The document’s interactive form (AcroForm) dictionary
    #[field("AcroForm")]
    acro_form: Option<TypedReference<'a, AcroForm<'a>>>,

    /// A metadata stream that shall contain metadata for the document
    ///
    /// Shall be an indirect reference
    #[field("Metadata")]
    metadata: Option<Reference>,

    /// The document’s structure tree root dictionary
    #[field("StructTreeRoot")]
    struct_tree_root: Option<TypedReference<'a, StructTreeRoot<'a>>>,

    /// A mark information dictionary that shall contain information about the
    /// document's usage of Tagged PDF conventions
    #[field("MarkInfo")]
    mark_info: Option<MarkInformationDictionary>,

    /// A language identifier that shall specify the natural language for all
    /// text in the document except where overridden by language specifications
    /// for structure elements or marked content.
    ///
    /// If this entry is absent, the language shall be considered unknown.
    #[field("Lang")]
    lang: Option<String>,

    /// A Web Capture information dictionary that shall contain state information
    /// used by any Web Capture extension
    #[field("SpiderInfo")]
    spider_info: Option<WebCapture>,

    /// An array of output intent dictionaries that shall specify the colour
    /// characteristics of output devices on which the document might be rendered
    #[field("OutputIntents")]
    output_intents: Option<Vec<OutputIntent>>,

    /// A page-piece dictionary associated with the document
    #[field("PieceInfo")]
    piece_info: Option<PagePiece<'a>>,

    /// The document's optional content properties dictionary
    ///
    /// Required if a document contains optional content
    #[field("OCProperties")]
    oc_properties: Option<OptionalContentProperties<'a>>,

    /// A permissions dictionary that shall specify user access permissions for
    /// the document.
    #[field("Perms")]
    perms: Option<Permissions>,

    /// A dictionary that shall contain attestations regarding the content of a
    /// PDF document, as it relates to the legality of digital signatures
    #[field("Legal")]
    legal: Option<Legal>,

    /// An array of requirement dictionaries that shall represent requirements
    /// for the document
    #[field("Requirements")]
    requirements: Option<Vec<Requirement>>,

    /// A collection dictionary that a conforming reader shall use to enhance
    /// the presentation of file attachments stored in the PDF document.
    #[field("Collection")]
    collection: Option<Collection>,

    /// A flag used to expedite the display of PDF documents containing XFA forms.
    /// It specifies whether the document shall be regenerated when the document
    /// is first opened
    ///
    /// Default value: false.
    #[field("NeedsRendering", default = false)]
    needs_rendering: bool,

    #[field("LastModified")]
    last_modified: Option<Date>,
}

#[derive(Debug, FromObj)]
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

    /// If the document was converted to PDF from another format, the name of the
    /// conforming product that created the original document from which it was
    /// converted
    #[field("Creator")]
    creator: Option<String>,

    /// If the document was converted to PDF from another format, the name of
    /// the conforming product that converted it to PDF
    #[field("Producer")]
    producer: Option<String>,

    #[field("CreationDate")]
    creation_date: Option<Date>,
    #[field("ModDate")]
    mod_date: Option<Date>,
    #[field("Trapped", default = Trapped::default())]
    trapped: Trapped,

    // todo: "other" field
    #[field("")]
    other: Dictionary<'a>,
}

/// A name object indicating whether the document has been modified to include
/// trapping information
#[pdf_enum]
#[derive(Default)]
pub enum Trapped {
    /// The document has been fully trapped; no further trapping shall be needed.
    /// This shall be the name "True", not the boolean value true.
    True = "True",

    /// The document has not yet been trapped. This shall be the name "False",
    /// not the boolean value false
    False = "False",

    /// Either it is unknown whether the document has been trapped or it has been
    /// partly but not yet fully trapped; some additional trapping may still be
    /// needed
    #[default]
    Unknown = "Unknown",
}

#[derive(Debug, FromObj)]
pub struct Extensions;
#[derive(Debug, FromObj)]
struct Language;

#[derive(Debug, FromObj)]
pub struct NameDictionary<'a> {
    /// A name tree mapping name strings to destinations
    #[field("Dests")]
    dests: Option<NameTree<'a>>,

    /// A name tree mapping name strings to annotation appearance streams
    #[field("AP")]
    ap: Option<NameTree<'a>>,

    /// A name tree mapping name strings to document-level JavaScript actions
    #[field("JavaScript")]
    java_script: Option<NameTree<'a>>,

    /// A name tree mapping name strings to visible pages for use in interactive
    /// forms
    #[field("Pages")]
    pages: Option<NameTree<'a>>,

    /// A name tree mapping name strings to invisible (template) pages for use
    /// in interactive forms
    #[field("Templates")]
    templates: Option<NameTree<'a>>,

    /// A name tree mapping digital identifiers to Web Capture content sets
    #[field("IDS")]
    ids: Option<NameTree<'a>>,

    /// A name tree mapping uniform resource locators (URLs) to Web Capture
    /// content sets
    #[field("URLS")]
    urls: Option<NameTree<'a>>,

    /// A name tree mapping name strings to file specifications for embedded file
    /// streams
    #[field("EmbeddedFiles")]
    embedded_files: Option<NameTree<'a>>,

    /// A name tree mapping name strings to alternate presentations
    #[field("AlternatePresentations")]
    alternate_presentations: Option<NameTree<'a>>,

    /// A name tree mapping name strings (which shall have Unicode encoding) to
    /// rendition objects
    #[field("Renditions")]
    renditions: Option<NameTree<'a>>,
}

#[derive(Debug, FromObj)]
pub struct NamedDestinations;
#[derive(Debug, FromObj)]
pub struct DocumentOutline;
#[derive(Debug, FromObj)]
pub struct ThreadDictionary;

pub fn assert_len(arr: &[Object], len: usize) -> PdfResult<()> {
    if arr.len() != len {
        anyhow::bail!(ParseError::ArrayOfInvalidLength {
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
            Object::Dictionary(dict) => {
                OpenAction::Actions(Actions::from_obj(Object::Dictionary(dict), lexer)?)
            }
            obj => OpenAction::Destination(Destination::from_obj(obj, lexer)?),
        })
    }
}

#[derive(Debug, FromObj)]
pub struct AdditionalActions;
#[derive(Debug, FromObj)]
pub struct UriDict;

#[derive(Debug, FromObj)]
pub struct AcroForm<'a> {
    /// An array of references to the document’s root fields (those with no
    /// ancestors in the field hierarchy).
    #[field("Fields")]
    fields: Vec<Reference>,

    /// A flag specifying whether to construct appearance streams and appearance
    /// dictionaries for all widget annotations in the document
    ///
    /// Default value: false
    #[field("NeedAppearances", default = false)]
    need_appearances: bool,

    /// A set of flags specifying various document-level characteristics related
    /// to signature fields
    ///
    /// Default value: 0
    #[field("SigFlags", default = SigFlags(0))]
    sig_flags: SigFlags,

    /// An array of indirect references to field dictionaries with calculation
    /// actions, defining the calculation order in which their values will be
    /// recalculated when the value of any field changes
    #[field("CO")]
    co: Option<Vec<Reference>>,

    /// A resource dictionary containing default resources (such as fonts, patterns,
    /// or colour spaces) that shall be used by form field appearance streams.
    /// At a minimum, this dictionary shall contain a Font entry specifying the
    /// resource name and font dictionary of the default font for displaying text.
    #[field("DR")]
    dr: Option<Dictionary<'a>>,

    /// A document-wide default value for the DA attribute of variable text fields
    #[field("DA")]
    da: Option<String>,

    /// A document-wide default value for the Q attribute of variable text fields
    #[field("Q")]
    q: Option<String>,

    /// A stream or array containing an XFA resource, whose format shall be
    /// described by the Data Package (XDP) Specification.
    ///
    /// The value of this entry shall be either a stream representing the entire
    /// contents of the XML Data Package or an array of text string and stream
    /// pairs representing the individual packets comprising the XML Data Package.
    // todo: struct for field
    #[field("XFA")]
    xfa: Option<String>,
}

#[derive(Debug, Clone)]
struct SigFlags(u32);

impl SigFlags {
    /// If set, the document contains at least one signature field. This flag
    /// allows a conforming reader to enable user interface items (such as menu
    /// items or pushbuttons) related to signature processing without having to
    /// scan the entire document for the presence of signature fields.
    pub const SIGNATURES_EXIST: u8 = 0b01;

    /// If set, the document contains signatures that may be invalidated if the
    /// file is saved (written) in a way that alters its previous contents, as
    /// opposed to an incremental update. Merely updating the file by appending
    /// new information to the end of the previous version is safe. Conforming
    /// readers may use this flag to inform a user requesting a full save that
    /// signatures will be invalidated and require explicit confirmation before
    /// continuing with the operation.
    pub const APPEND_ONLY: u8 = 0b10;
}

impl<'a> FromObj<'a> for SigFlags {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        Ok(Self(u32::from_obj(obj, resolver)?))
    }
}

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
    /// If Suspects is true, the document may not completely conform to Tagged
    /// PDF conventions.
    ///
    /// Default value: false
    #[field("Marked", default = false)]
    marked: bool,

    /// A flag indicating the presence of structure elements that contain user
    /// properties attributes
    ///
    /// Default value: false
    #[field("UserProperties", default = false)]
    user_properties: bool,

    /// A flag indicating the presence of tag suspects
    ///
    /// Default value: false
    #[field("Suspects", default = false)]
    suspects: bool,
}

#[derive(Debug, FromObj)]
pub struct WebCapture;
#[derive(Debug, FromObj)]
pub struct OutputIntent;

#[derive(Debug, Clone)]
pub struct PagePiece<'a>(Dictionary<'a>);

impl<'a> FromObj<'a> for PagePiece<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = resolver.assert_dict(obj)?;
        Ok(Self(dict))
    }
}

#[derive(Debug, FromObj)]
pub struct Permissions;
#[derive(Debug, FromObj)]
pub struct Legal;
#[derive(Debug, FromObj)]
pub struct Requirement;
#[derive(Debug, Clone, PartialEq, FromObj)]
pub struct Collection;
#[derive(Debug, FromObj)]
pub struct BoxColorInfo;

#[derive(Debug, Clone, FromObj)]
#[obj_type("Group")]
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

#[derive(Debug, FromObj)]
pub struct Transitions;
#[derive(Debug, FromObj)]
pub struct SeparationInfo;
#[derive(Debug, FromObj)]
pub struct NavigationNode;
#[derive(Debug, FromObj)]
pub struct Viewport;
#[derive(Debug, FromObj)]
pub struct PropertyList;

/// Specifies the page layout when the document is opened
#[pdf_enum]
#[derive(Default)]
enum PageLayout {
    /// Display one page at a time
    #[default]
    SinglePage = "SinglePage",

    /// Display the pages in one column
    OneColumn = "OneColumn",

    /// Display the pages in two columns, with odd-numbered pages on the left
    TwoColumnLeft = "TwoColumnLeft",

    /// Display the pages in two columns, with odd-numbered pages on the right
    TwoColumnRight = "TwoColumnRight",

    /// Display the pages two at a time, with odd-numbered pages on the left
    TwoPageLeft = "TwoPageLeft",

    /// Display the pages two at a time, with odd-numbered pages on the right
    TwoPageRight = "TwoPageRight",
}

/// A name object specifying how the document shall be displayed when opened
#[pdf_enum]
#[derive(Default)]
pub enum PageMode {
    /// Neither document outline nor thumbnail images visible
    #[default]
    UseNone = "UseNone",

    /// Document outline visible
    UseOutlines = "UseOutlines",

    /// Thumbnail images visible
    UseThumbs = "UseThumbs",

    /// Full-screen mode, with no menu bar, window controls, or any other window
    /// visible
    FullScreen = "FullScreen",

    /// Optional content group panel visible
    UseOc = "UseOc",

    /// Attachments panel visible
    UseAttachments = "UseAttachments",
}
