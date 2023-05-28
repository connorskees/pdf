use crate::{catalog::PageMode, error::PdfResult, objects::Object, FromObj, Resolve};

#[derive(Debug, FromObj)]
pub struct ViewerPreferences {
    /// A flag specifying whether to hide the conforming reader's tool bars when
    /// the document is active.
    ///
    /// Default value: false
    #[field("HideToolbar", default = false)]
    hide_toolbar: bool,

    /// A flag specifying whether to hide the conforming reader's menu bar when
    /// the document is active.
    ///
    /// Default value: false
    #[field("HideMenubar", default = false)]
    hide_menubar: bool,

    /// A flag specifying whether to hide user interface elements in the document's
    /// window (such as scroll bars and navigation controls), leaving only the
    /// document's contents displayed.
    ///
    /// Default value: false
    #[field("HideWindowUI", default = false)]
    hide_window_ui: bool,

    /// A flag specifying whether to resize the document's window to fit the size
    /// of the first displayed page.
    ///
    /// Default value: false
    #[field("FitWindow", default = false)]
    fit_window: bool,

    /// A flag specifying whether to position the document's window in the center
    /// of the screen.
    ///
    /// Default value: false
    #[field("CenterWindow", default = false)]
    center_window: bool,

    /// A flag specifying whether the window's title bar should display the document
    /// title taken from the Title entry of the document information dictionary.
    /// If false, the title bar should instead display the name of the PDF file
    /// containing the document.
    ///
    /// Default value: false
    #[field("DisplayDocTitle", default = false)]
    display_doc_title: bool,

    /// The document's page mode, specifying how to display the document on exiting
    /// full-screen mode
    ///
    /// This entry is meaningful only if the value of the PageMode entry in the
    /// Catalog dictionary is FullScreen;
    /// it shall be ignored otherwise.
    ///
    /// Default value: UseNone.
    #[field("NonFullScreenPageMode", default = PageMode::default())]
    non_full_screen_page_mode: PageMode,

    /// The predominant reading order for text
    ///
    /// This entry has no direct effect on the document's contents or page numbering
    /// but may be used to determine the relative positioning of pages when displayed
    /// side by side or printed n-up.
    ///
    /// Default value: L2R
    #[field("Direction", default = TextDirection::default())]
    direction: TextDirection,

    /// The name of the page boundary representing the area of a page that shall
    /// be displayed when viewing the document on the screen. The value is the
    /// key designating the relevant page boundary in the page object. If the
    /// specified page boundary is not defined in the page object, its default
    /// value shall be used.
    ///
    /// Default value: CropBox.
    ///
    /// This entry is intended primarily for use by prepress applications that
    /// interpret or manipulate the page boundaries. Most conforming readers
    /// disregard it
    #[field("ViewArea", default = PageBoundary::default())]
    view_area: PageBoundary,

    /// The name of the page boundary to which the contents of a page shall be
    /// clipped when viewing the document on the screen. The value is the key
    /// designating the relevant page boundary in the page object. If the specified
    /// page boundary is not defined in the page object, its default value shall
    /// be used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that
    /// interpret or manipulate the page boundaries. Most conforming readers
    /// disregard it
    #[field("ViewClip", default = PageBoundary::default())]
    view_clip: PageBoundary,

    /// The name of the page boundary representing the area of a page that shall
    /// be rendered when printing the document. The value is the key designating
    /// the relevant page boundary in the page object. If the specified page
    /// boundary is not defined in the page object, its default value shall be
    /// used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that
    /// interpret or manipulate the page boundaries. Most conforming readers
    /// disregard it
    #[field("PrintArea", default = PageBoundary::default())]
    print_area: PageBoundary,

    /// The name of the page boundary to which the contents of a page shall be
    /// clipped when printing the document. The value is the key designating the
    /// relevant page boundary in the page object. If the specified page boundary
    /// is not defined in the page object, its default value shall be used
    ///
    /// Default value: CropBox
    ///
    /// This entry is intended primarily for use by prepress applications that
    /// interpret or manipulate the page boundaries. Most conforming readers
    /// disregard it
    #[field("PrintClip", default = PageBoundary::default())]
    print_clip: PageBoundary,

    /// The page scaling option that shall be selected when a print dialog is
    /// displayed for this document.
    ///
    /// Valid values are
    ///   * None, which indicates no page scaling
    ///   * AppDefault, which indicates the conforming reader's default print scaling.
    ///
    /// If this entry has an unrecognized value, AppDefault shall be used.
    ///
    /// Default value: AppDefault.
    ///
    /// If the print dialog is suppressed and its parameters are provided from
    /// some other source, this entry nevertheless shall be honored
    #[field("PrintScaling", default = PageScaling::default())]
    print_scaling: PageScaling,

    /// The paper handling option that shall be used when printing the file from
    /// the print dialog
    #[field("Duplex")]
    duplex: Option<Duplex>,

    /// A flag specifying whether the PDF page size shall be used to select the
    /// input paper tray. This setting influences only the preset values used
    /// to populate the print dialog presented by a conforming reader. If
    /// PickTrayByPDFSize is true, the check box in the print dialog associated
    /// with input paper tray shall be checked.
    ///
    /// This setting has no effect on operating systems that do not provide the
    /// ability to pick the input tray by size.
    ///
    /// Default value: as defined by the conforming reader
    #[field("PickTrayByPDFSize", default = false)]
    pick_tray_by_pdf_size: bool,

    /// The page numbers used to initialize the print dialog box when the file
    /// is printed. The array shall contain an even number of integers to be
    /// interpreted in pairs, with each pair specifying the first and last pages
    /// in a sub-range of pages to be printed.The first page of the PDF file shall
    /// be denoted by 1.
    ///
    /// Default value: as defined by the conforming reader
    #[field("PrintPageRange")]
    print_page_range: Option<PageRanges>,

    /// The number of copies that shall be printed when the print dialog is opened
    /// for this file. Values outside this range shall be ignored.
    ///
    /// Default value: as defined by the conforming reader, but typically 1
    #[field("NumCopies")]
    num_copies: Option<u32>,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
struct PageRanges(Vec<PageRange>);

#[derive(Debug, Copy, Clone)]
struct PageRange {
    first: u32,
    last: u32,
}

impl<'a> FromObj<'a> for PageRanges {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let arr = resolver.assert_arr(obj)?;
        let ranges = arr
            .chunks_exact(2)
            .map(|objs| {
                let first = resolver.assert_unsigned_integer(objs[0].clone())?;
                let last = resolver.assert_unsigned_integer(objs[1].clone())?;

                Ok(PageRange { first, last })
            })
            .collect::<PdfResult<Vec<PageRange>>>()?;

        Ok(PageRanges(ranges))
    }
}

#[pdf_enum]
#[derive(Default)]
enum TextDirection {
    #[default]
    LeftToRight = "L2R",

    /// Right to left (including vertical writing systems, such as Chinese,
    /// Japanese, and Korean)
    RightToLeft = "R2L",
}

#[pdf_enum]
#[derive(Default)]
enum PageScaling {
    #[default]
    AppDefault = "AppDefault",
    None = "None",
}

#[pdf_enum]
enum Duplex {
    /// Print single-sided
    Simplex = "Simplex",

    /// Duplex and flip on the short edge of the sheet
    DuplexFlipShortEdge = "DuplexFlipShortEdge",

    /// Duplex and flip on the long edge of the sheet
    DuplexFlipLongEdge = "DuplexFlipLongEdge",
}

#[pdf_enum]
#[derive(Default)]
enum PageBoundary {
    MediaBox = "MediaBox",
    #[default]
    CropBox = "CropBox",
    BleedBox = "BleedBox",
    TrimBox = "TrimBox",
    ArtBox = "ArtBox",
}
