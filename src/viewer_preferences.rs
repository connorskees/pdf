use crate::{
    catalog::PageMode,
    error::PdfResult,
    objects::{Dictionary, Object},
    pdf_enum, Resolve,
};

#[derive(Debug)]
pub struct ViewerPreferences {
    /// A flag specifying whether to hide the conforming reader's tool bars when
    /// the document is active.
    ///
    /// Default value: false
    hide_toolbar: bool,

    /// A flag specifying whether to hide the conforming reader's menu bar when
    /// the document is active.
    ///
    /// Default value: false
    hide_menubar: bool,

    /// A flag specifying whether to hide user interface elements in the document's
    /// window (such as scroll bars and navigation controls), leaving only the
    /// document's contents displayed.
    ///
    /// Default value: false
    hide_window_ui: bool,

    /// A flag specifying whether to resize the document's window to fit the size
    /// of the first displayed page.
    ///
    /// Default value: false
    fit_window: bool,

    /// A flag specifying whether to position the document's window in the center
    /// of the screen.
    ///
    /// Default value: false
    center_window: bool,

    /// A flag specifying whether the window's title bar should display the document
    /// title taken from the Title entry of the document information dictionary.
    /// If false, the title bar should instead display the name of the PDF file
    /// containing the document.
    ///
    /// Default value: false
    display_doc_title: bool,

    /// The document's page mode, specifying how to display the document on exiting
    /// full-screen mode
    ///
    /// This entry is meaningful only if the value of the PageMode entry in the
    /// Catalog dictionary is FullScreen;
    /// it shall be ignored otherwise.
    ///
    /// Default value: UseNone.
    non_full_screen_page_mode: PageMode,

    /// The predominant reading order for text
    ///
    /// This entry has no direct effect on the document's contents or page numbering
    /// but may be used to determine the relative positioning of pages when displayed
    /// side by side or printed n-up.
    ///
    /// Default value: L2R
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
    print_scaling: PageScaling,

    /// The paper handling option that shall be used when printing the file from
    /// the print dialog
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
    pick_tray_by_pdf_size: bool,

    /// The page numbers used to initialize the print dialog box when the file
    /// is printed. The array shall contain an even number of integers to be
    /// interpreted in pairs, with each pair specifying the first and last pages
    /// in a sub-range of pages to be printed.The first page of the PDF file shall
    /// be denoted by 1.
    ///
    /// Default value: as defined by the conforming reader
    print_page_range: Option<Vec<PageRange>>,

    /// The number of copies that shall be printed when the print dialog is opened
    /// for this file. Values outside this range shall be ignored.
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

        /// Right to left (including vertical writing systems, such as Chinese,
        /// Japanese, and Korean)
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
