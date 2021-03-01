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

use std::collections::HashMap;

use crate::{
    actions::Actions,
    assert_empty, assert_reference,
    data_structures::NumberTree,
    date::Date,
    graphics_state_parameters::GraphicsStateParameters,
    objects::{ObjectType, TypeOrArray},
    pdf_enum,
    structure::StructTreeRoot,
    Dictionary, Lexer, Object, ParseError, PdfResult, Reference, Resolve,
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

    /// The page tree node that shall be the root of the document’s
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

    /// The document’s name dictionary
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
pub struct ViewerPreferences {
    /// A flag specifying whether to hide the conforming reader’s tool bars when the document is active.
    ///
    /// Default value: false
    hide_toolbar: bool,

    /// A flag specifying whether to hide the conforming reader’s menu bar when the document is active.
    ///
    /// Default value: false
    hide_menubar: bool,

    /// A flag specifying whether to hide user interface elements in the document’s window (such as scroll
    /// bars and navigation controls), leaving only the document’s contents displayed.
    ///
    /// Default value: false
    hide_window_ui: bool,

    /// A flag specifying whether to resize the document’s window to fit the size of the first displayed page.
    ///
    /// Default value: false
    fit_window: bool,

    /// A flag specifying whether to position the document’s window in the center of the screen.
    ///
    /// Default value: false
    center_window: bool,

    /// A flag specifying whether the window’s title bar should display the document title taken from the Title
    /// entry of the document information dictionary. If false, the title bar should instead display the name
    /// of the PDF file containing the document.
    ///
    /// Default value: false
    display_doc_title: bool,

    /// The document’s page mode, specifying how to display the document on exiting full-screen mode
    ///
    /// This entry is meaningful only if the value of the PageMode entry in the Catalog dictionary is FullScreen;
    /// it shall be ignored otherwise. Default value: UseNone.
    non_full_screen_page_mode: PageMode,

    /// The predominant reading order for text
    ///
    /// This entry has no direct effect on the document’s contents or page numbering but may be used to determine
    /// the relative positioning of pages when displayed side by side or printed n-up.
    ///
    /// Default value: L2R
    direction: TextDirection,

    /// The name of the page boundary representing the area of a page that shall be displayed when viewing the
    /// document on the screen. The value is the key designating the relevant page boundary in the page object.
    /// If the specified page boundary is not defined in the page object, its default value shall be used.
    ///
    /// Default value: CropBox.
    ///
    /// This entry is intended primarily for use by prepress applications that interpret or manipulate the page
    /// boundaries. Most conforming readers disregard it
    view_area: PageBoundary,

    /// The name of the page boundary to which the contents of a page shall be clipped when viewing the document
    /// on the screen. The value is the key designating the relevant page boundary in the page object. If the
    /// specified page boundary is not defined in the page object, its default value shall be used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that interpret or manipulate the page
    /// boundaries. Most conforming readers disregard it
    view_clip: PageBoundary,

    /// The name of the page boundary representing the area of a page that shall be rendered when printing the
    /// document. The value is the key designating the relevant page boundary in the page object. If the specified
    /// page boundary is not defined in the page object, its default value shall be used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that interpret or manipulate the page
    /// boundaries. Most conforming readers disregard it
    print_area: PageBoundary,

    /// The name of the page boundary to which the contents of a page shall be clipped when printing the document.
    /// The value is the key designating the relevant page boundary in the page object. If the specified page
    /// boundary is not defined in the page object, its default value shall be used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that interpret or manipulate the page
    /// boundaries. Most conforming readers disregard it
    print_clip: PageBoundary,

    /// The page scaling option that shall be selected when a print dialog is displayed for this document.
    /// Valid values are
    ///   * None, which indicates no page scaling
    ///   * AppDefault, which indicates the conforming reader’s default print scaling.
    ///
    /// If this entry has an unrecognized value, AppDefault shall be used.
    ///
    /// Default value: AppDefault.
    ///
    /// If the print dialog is suppressed and its parameters are provided from some other source, this entry
    /// nevertheless shall be honored
    print_scaling: PageScaling,

    /// The paper handling option that shall be used when printing the file from the print dialog
    duplex: Option<Duplex>,

    /// A flag specifying whether the PDF page size shall be used to select the input paper tray. This setting
    /// influences only the preset values used to populate the print dialog presented by a conforming reader. If
    /// PickTrayByPDFSize is true, the check box in the print dialog associated with input paper tray shall be checked.
    ///
    /// This setting has no effect on operating systems that do not provide the ability to pick the input tray by size.
    ///
    /// Default value: as defined by the conforming reader
    pick_tray_by_pdf_size: bool,

    /// The page numbers used to initialize the print dialog box when the file is printed. The array shall contain an
    /// even number of integers to be interpreted in pairs, with each pair specifying the first and last pages in a
    /// sub-range of pages to be printed.The first page of the PDF file shall be denoted by 1.
    ///
    /// Default value: as defined by the conforming reader
    print_page_range: Option<Vec<PageRange>>,

    /// The number of copies that shall be printed when the print dialog is opened for this file. Values outside this
    /// range shall be ignored.
    ///
    /// Default value: as defined by the conforming reader, but typically 1
    num_copies: Option<u32>,
}

impl ViewerPreferences {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let hide_toolbar = dict.get_bool("HideToolbar", resolver)?.unwrap_or(false);
        let hide_menubar = dict.get_bool("HideMenubar", resolver)?.unwrap_or(false);
        let hide_window_ui = dict.get_bool("HideWindowUI", resolver)?.unwrap_or(false);
        let fit_window = dict.get_bool("FitWindow", resolver)?.unwrap_or(false);
        let center_window = dict.get_bool("CenterWindow", resolver)?.unwrap_or(false);
        let display_doc_title = dict.get_bool("DisplayDocTitle", resolver)?.unwrap_or(false);

        let non_full_screen_page_mode = dict
            .get_name("NonFullScreenPageMode", resolver)?
            .as_deref()
            .map(PageMode::from_str)
            .transpose()?
            .unwrap_or_default();

        let direction = dict
            .get_name("Direction", resolver)?
            .as_deref()
            .map(TextDirection::from_str)
            .transpose()?
            .unwrap_or_default();

        let view_area = dict
            .get_name("ViewArea", resolver)?
            .as_deref()
            .map(PageBoundary::from_str)
            .transpose()?
            .unwrap_or_default();

        let view_clip = dict
            .get_name("ViewClip", resolver)?
            .as_deref()
            .map(PageBoundary::from_str)
            .transpose()?
            .unwrap_or_default();

        let print_area = dict
            .get_name("PrintArea", resolver)?
            .as_deref()
            .map(PageBoundary::from_str)
            .transpose()?
            .unwrap_or_default();

        let print_clip = dict
            .get_name("PrintClip", resolver)?
            .as_deref()
            .map(PageBoundary::from_str)
            .transpose()?
            .unwrap_or_default();

        let print_scaling = dict
            .get_name("PrintScaling", resolver)?
            .as_deref()
            .map(PageScaling::from_str)
            .transpose()?
            .unwrap_or_default();

        let duplex = dict
            .get_name("Duplex", resolver)?
            .as_deref()
            .map(Duplex::from_str)
            .transpose()?;

        let pick_tray_by_pdf_size = dict
            .get_bool("PickTrayByPDFSize", resolver)?
            .unwrap_or(false);

        let print_page_range = dict
            .get_arr("PrintPageRange", resolver)?
            .map(|objs| {
                objs.chunks_exact(2)
                    .map(|objs| PageRange::from_objs(objs[0].clone(), objs[1].clone(), resolver))
                    .collect::<PdfResult<Vec<PageRange>>>()
            })
            .transpose()?;

        let num_copies = dict.get_unsigned_integer("NumCopies", resolver)?;

        Ok(Self {
            hide_toolbar,
            hide_menubar,
            hide_window_ui,
            fit_window,
            center_window,
            display_doc_title,
            non_full_screen_page_mode,
            direction,
            view_area,
            view_clip,
            print_area,
            print_clip,
            print_scaling,
            duplex,
            pick_tray_by_pdf_size,
            print_page_range,
            num_copies,
        })
    }
}

#[derive(Debug)]
struct PageRange {
    first: u32,
    last: u32,
}

impl PageRange {
    pub fn from_objs(first: Object, last: Object, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let first = resolver.assert_unsigned_integer(first)?;
        let last = resolver.assert_unsigned_integer(last)?;

        Ok(PageRange { first, last })
    }
}

pdf_enum!(
    #[derive(Debug)]
    enum TextDirection {
        LeftToRight = "L2R",

        /// Right to left (including vertical writing systems, such as Chinese, Japanese, and Korean)
        RightToLeft = "R2L",
    }
);

impl Default for TextDirection {
    fn default() -> Self {
        Self::LeftToRight
    }
}

pdf_enum!(
    #[derive(Debug)]
    enum PageScaling {
        AppDefault = "AppDefault",
        None = "None",
    }
);

impl Default for PageScaling {
    fn default() -> Self {
        Self::AppDefault
    }
}

pdf_enum!(
    #[derive(Debug)]
    enum Duplex {
        /// Print single-sided
        Simplex = "Simplex",

        /// Duplex and flip on the short edge of the sheet
        DuplexFlipShortEdge = "DuplexFlipShortEdge",

        /// Duplex and flip on the long edge of the sheet
        DuplexFlipLongEdge = "DuplexFlipLongEdge",
    }
);

pdf_enum!(
    #[derive(Debug)]
    enum PageBoundary {
        MediaBox = "MediaBox",
        CropBox = "CropBox",
        BleedBox = "BleedBox",
        TrimBox = "TrimBox",
        ArtBox = "ArtBox",
    }
);

impl Default for PageBoundary {
    fn default() -> Self {
        Self::CropBox
    }
}

#[derive(Debug)]
pub struct DocumentOutline;
#[derive(Debug)]
pub struct ThreadDictionary;

#[derive(Debug)]
enum DestinationKind {
    /// Display the page designated by page, with the coordinates (left, top) positioned
    /// at the upper-left corner of the window and the contents of the page magnified by
    /// the factor zoom. A null value for any of the parameters left, top, or zoom specifies
    /// that the current value of that parameter shall be retained unchanged. A zoom value
    /// of 0 has the same meaning as a null value.
    Xyz {
        left: Option<f32>,
        top: Option<f32>,
        zoom: Option<f32>,
    },

    /// Display the page designated by page, with its contents magnified just enough to fit
    /// the entire page within the window both horizontally and vertically. If the required
    /// horizontal and vertical magnification factors are different, use the smaller of the
    /// two, centering the page within the window in the other dimension.
    Fit,

    /// Display the page designated by page, with the vertical coordinate top positioned at
    /// the top edge of the window and the contents of the page magnified just enough to fit
    /// the entire width of the page within the window. A null value for top specifies that
    /// the current value of that parameter shall be retained unchanged.
    FitH { top: Option<f32> },

    /// Display the page designated by page, with the horizontal coordinate left positioned
    /// at the left edge of the window and the contents of the page magnified just enough to
    /// fit the entire height of the page within the window. A null value for left specifies
    /// that the current value of that parameter shall be retained unchanged.
    FitV { left: Option<f32> },

    /// Display the page designated by page, with its contents magnified just enough to fit
    /// the rectangle specified by the coordinates left, bottom, right, and top entirely within
    /// the window both horizontally and vertically. If the required horizontal and vertical
    /// magnification factors are different, use the smaller of the two, centering the rectangle
    /// within the window in the other dimension.
    FitR {
        left: Option<f32>,
        bottom: Option<f32>,
        right: Option<f32>,
        top: Option<f32>,
    },

    /// Display the page designated by page, with its contents magnified just enough to fit its
    /// bounding box entirely within the window both horizontally and vertically. If the required
    /// horizontal and vertical magnification factors are different, use the smaller of the two,
    /// centering the bounding box within the window in the other dimension.
    FitB,

    /// Display the page designated by page, with the vertical coordinate top positioned at the
    /// top edge of the window and the contents of the page magnified just enough to fit the entire
    /// width of its bounding box within the window. A null value for top specifies that the
    /// current value of that parameter shall be retained unchanged.
    FitBh { top: Option<f32> },

    /// Display the page designated by page, with the horizontal coordinate left positioned at the
    /// left edge of the window and the contents of the page magnified just enough to fit the entire
    /// height of its bounding box within the window. A null value for left specifies that the
    /// current value of that parameter shall be retained unchanged.
    FitBv { left: Option<f32> },
}

#[derive(Debug)]
pub struct Destination {
    kind: DestinationKind,
    page_ref: Reference,
}

pub fn assert_len(arr: &[Object], len: usize) -> PdfResult<()> {
    if arr.len() != len {
        return Err(ParseError::ArrayOfInvalidLength {
            expected: len,
            found: arr.to_vec(),
        });
    }

    Ok(())
}

impl Destination {
    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut impl Resolve) -> PdfResult<Self> {
        if arr.len() < 2 {
            return Err(ParseError::ArrayOfInvalidLength {
                expected: 2,
                found: arr,
            });
        }

        let vals = arr.split_off(2);

        let dimensions = vals
            .iter()
            .cloned()
            .map(|obj| resolver.assert_or_null(obj, Resolve::assert_number))
            .collect::<PdfResult<Vec<Option<f32>>>>()?;

        let kind_str = resolver.assert_name(arr.pop().unwrap())?;

        let page_ref = assert_reference(arr.pop().unwrap())?;

        let kind = match kind_str.as_str() {
            "XYZ" => {
                assert_len(&vals, 3)?;
                DestinationKind::Xyz {
                    left: dimensions[0],
                    top: dimensions[1],
                    zoom: dimensions[2],
                }
            }
            "Fit" => {
                assert_len(&vals, 0)?;
                DestinationKind::Fit
            }
            "FitH" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitH { top: dimensions[0] }
            }
            "FitV" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitV {
                    left: dimensions[0],
                }
            }
            "FitR" => {
                assert_len(&vals, 4)?;
                DestinationKind::FitR {
                    left: dimensions[0],
                    bottom: dimensions[1],
                    right: dimensions[2],
                    top: dimensions[3],
                }
            }
            "FitB" => {
                assert_len(&vals, 0)?;
                DestinationKind::FitB
            }
            "FitBH" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitBh { top: dimensions[0] }
            }
            "FitBV" => {
                assert_len(&vals, 1)?;
                DestinationKind::FitBv {
                    left: dimensions[0],
                }
            }
            found => {
                return Err(ParseError::UnrecognizedVariant {
                    found: found.to_owned(),
                    ty: "DestinationKey",
                })
            }
        };

        Ok(Destination { kind, page_ref })
    }
}

#[derive(Debug)]
pub enum OpenAction {
    Destination(Destination),
    Actions(Actions),
}

impl OpenAction {
    pub fn from_obj(obj: Object, lexer: &mut Lexer) -> PdfResult<Self> {
        Ok(match obj {
            Object::Array(arr) => OpenAction::Destination(Destination::from_arr(arr, lexer)?),
            Object::Dictionary(dict) => OpenAction::Actions(Actions::from_dict(dict, lexer)?),
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    found,
                    expected: &[ObjectType::Array, ObjectType::Dictionary],
                })
            }
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
#[derive(Debug)]
pub struct OptionalContentProperties;
#[derive(Debug)]
pub struct Permissions;
#[derive(Debug)]
pub struct Legal;
#[derive(Debug)]
pub struct Requirement;
#[derive(Debug, Clone)]
pub struct Collection;

#[derive(Debug)]
pub struct Resources {
    /// A dictionary that maps resource names to
    /// graphics state parameter dictionaries
    ext_g_state: Option<HashMap<String, GraphicsStateParameters>>,

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
    pub(crate) fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let ext_g_state = dict
            .get_dict("ExtGState", lexer)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, value)| {
                        let dict = lexer.assert_dict(value)?;
                        let graphics = GraphicsStateParameters::from_dict(dict, lexer)?;

                        Ok((key, graphics))
                    })
                    .collect::<PdfResult<HashMap<String, GraphicsStateParameters>>>()
            })
            .transpose()?;

        let color_space = dict.get_dict("ColorSpace", lexer)?;
        let pattern = dict.get_dict("Pattern", lexer)?;
        let shading = dict.get_dict("Shading", lexer)?;
        let xobject = dict.get_dict("XObject", lexer)?;
        let font = dict.get_dict("Font", lexer)?;

        let proc_set = dict
            .get_arr("ProcSet", lexer)?
            .map(|proc| {
                proc.into_iter()
                    .map(|proc| ProcedureSet::from_str(&lexer.assert_name(proc)?))
                    .collect::<PdfResult<Vec<ProcedureSet>>>()
            })
            .transpose()?;
        let properties = dict.get_dict("Properties", lexer)?;

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
pub struct Rectangle {
    lower_left_x: f32,
    lower_left_y: f32,
    upper_right_x: f32,
    upper_right_y: f32,
}

impl Rectangle {
    pub(crate) fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 4)?;

        let upper_right_y = resolver.assert_number(arr.pop().unwrap())?;
        let upper_right_x = resolver.assert_number(arr.pop().unwrap())?;
        let lower_left_y = resolver.assert_number(arr.pop().unwrap())?;
        let lower_left_x = resolver.assert_number(arr.pop().unwrap())?;

        Ok(Rectangle {
            lower_left_x,
            lower_left_y,
            upper_right_x,
            upper_right_y,
        })
    }
}

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
    /// group or page from which to inherit—in particular, one that is the value of the G entry
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
    cs: Option<TypeOrArray<Object>>,

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
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<Self> {
        let s = dict.expect_name("S", lexer)?;
        if s != "Transparency" {
            todo!()
        }

        let cs = dict.get_type_or_arr("CS", lexer, |_, obj| Ok(obj))?;
        let is_isolated = dict.get_bool("I", lexer)?.unwrap_or(false);
        let is_knockout = dict.get_bool("K", lexer)?.unwrap_or(false);

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
pub struct Pattern;
#[derive(Debug)]
pub struct Shading;
#[derive(Debug)]
pub struct XObject;
#[derive(Debug)]
pub struct Font;
#[derive(Debug)]
pub struct PropertyList;

pdf_enum!(
    #[derive(Debug)]
    pub enum ProcedureSet {
        Pdf = "PDF",
        Text = "Text",
        ImageB = "ImageB",
        ImageC = "ImageC",
        ImageI = "ImageI",
    }
);

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
    enum PageMode {
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
