/*!
 * https://www.pdfa.org/norm-refs/5620.PortableJobTicket.pdf
 */

use crate::{
    catalog::Trapped,
    data_structures::Rectangle,
    date::Date,
    objects::{Dictionary, Object},
};

#[derive(Debug, Clone, FromObj)]
pub struct JobTicket<'a> {
    #[field("A")]
    audit: Vec<Audit<'a>>,
    #[field("Cn")]
    contents: Option<[JobTicketContents<'a>; 1]>,

    /// An array of PDF objects. These objects may be PDF resources, or they may be
    /// resources defined in compliance with some other PDL. If not PDF
    /// resources, they must be PDF stream objects with the following keys in
    /// their stream dictionarie
    #[field("R")]
    resources: Option<Vec<Object<'a>>>,
    #[field("RA")]
    resource_alias: Option<Vec<ResourceAlias>>,

    /// Lowest version of the job ticket specification with which this particular job
    /// ticket complies. Version implies that there are no objects or keys which
    /// appear in the job ticket which come from a higher version of the job
    /// ticket specification
    #[field("V")]
    version: f32,
}

/// Audit objects keep track of changes in a job ticket. They are useful in multistep
/// production work environments
#[derive(Debug, Clone, FromObj)]
struct Audit<'a> {
    /// An Address object for the person responsible for this change of the job
    /// ticket
    #[field("Au")]
    author: Option<Address>,

    /// A comment describing details of the action documented by this Audit object
    #[field("C")]
    comment: Option<String>,

    /// Date on which the action was concluded
    #[field("Dt")]
    date: Date,

    /// Arbitrary key/value pairs describing details of the action documented by this
    /// Audit object
    #[field("D")]
    details: Option<Dictionary<'a>>,

    /// An array of JTFile objects which reference the physical files affected by the
    /// operation recorded by this Audit object
    // todo: better type?
    #[field("Fi")]
    files: Option<Vec<JTFile<'a>>>,

    /// Identifies the Job Ticket Manager that acted on the job ticket. The name of
    /// the process or application that created the file
    #[field("JTM")]
    job_ticket_manager: String,
}

#[derive(Debug, Clone, FromObj)]
#[obj_type("JobTicketContents")]
struct JobTicketContents<'a> {
    #[field("A")]
    accounting: Option<Accounting>,
    #[field("Ad")]
    administrator: Option<Address>,
    #[field("Bl")]
    bleed_box: Option<Rectangle>,
    #[field("Co")]
    colorant_control: Option<ColorantControl>,
    #[field("Cm")]
    comments: Option<String>,
    #[field("Dl")]
    delivery: Option<Vec<Delivery>>,
    #[field("D")]
    documents: Option<Vec<Document<'a>>>,
    #[field("Em")]
    end_message: Option<String>,
    #[field("F")]
    finishing: Option<Vec<Finishing>>,
    #[field("FP")]
    font_policy: Option<FontPolicy>,
    #[field("IH")]
    ignore_halftone: Option<bool>,
    #[field("IPD")]
    ignore_page_device: Option<bool>,
    #[field("IP")]
    insert_page: Option<InsertPage>,
    #[field("IS")]
    insert_sheet: Option<InsertSheet>,
    #[field("JN")]
    job_name: Option<String>,
    #[field("L")]
    layout: Option<Layout>,
    #[field("MB")]
    media_box: Option<Rectangle>,
    #[field("MD")]
    mark_documents: Option<Vec<Document<'a>>>,
    #[field("MS")]
    media_source: Option<MediaSource>,
    #[field("MU")]
    media_usage: Option<MediaUsage>,
    #[field("Ns")]
    new_sheet: Option<InsertSheet>,
    #[field("PL")]
    print_layout: Option<PrintLayout>,
    #[field("R")]
    rendering: Option<Rendering<'a>>,
    #[field("Sc")]
    scheduling: Option<Scheduling>,
    #[field("SM")]
    start_message: Option<String>,
    #[field("S")]
    submitter: Option<Address>,
    #[field("T")]
    trapping: Option<Trapping>,
    #[field("TB")]
    trim_box: Option<Rectangle>,
    #[field("Tl")]
    trailer: Option<InsertSheet>,
    #[field("TD")]
    trapping_description: Option<String>,
    #[field("TP")]
    trapping_parameters: Option<TrappingParameters>,
    #[field("TR")]
    trap_regions: Option<Vec<TrapRegion>>,
    #[field("TSS")]
    trapping_source_selector: Option<TrappingSourceSelector>,
}

#[derive(Debug, Clone, FromObj)]
struct ResourceAlias {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Address {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Accounting {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct ColorantControl {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Delivery {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Document<'a> {
    #[field("Bl")]
    bleed_box: Option<Rectangle>,
    #[field("Co")]
    colorant_control: Option<ColorantControl>,
    #[field("Cm")]
    comments: Option<String>,
    #[field("Cp")]
    copies: Option<i32>,
    #[field("Fi")]
    files: Option<Vec<JTFile<'a>>>,
    #[field("IP")]
    insert_page: Option<InsertPage>,
    #[field("IS")]
    insert_sheet: Option<InsertSheet>,
    #[field("MB")]
    media_box: Option<Rectangle>,
    #[field("Na")]
    name: Option<String>,
    #[field("Ns")]
    new_sheet: Option<InsertSheet>,
    #[field("P")]
    pages: Option<Vec<PageRange<'a>>>,
    #[field("R")]
    rendering: Option<Rendering<'a>>,
    #[field("T")]
    trapping: Option<Trapping>,
    #[field("TB")]
    trim_box: Option<Rectangle>,
    #[field("Tl")]
    trailer: Option<InsertSheet>,
    #[field("TD")]
    trapping_description: Option<String>,
    #[field("TP")]
    trapping_parameters: Option<TrappingParameters>,
    #[field("TR")]
    trap_regions: Option<Vec<TrapRegion>>,
}

#[derive(Debug, Clone, FromObj)]
struct Finishing {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct FontPolicy {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct InsertPage {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct InsertSheet {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Layout {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct MediaSource {
    /// Product-specific classification of media, which may influence rendering. For
    /// example, transparent or glossy media may affect the selection of a color
    /// rendering method or a post-rendering technique specific to the device
    #[field("Cl")]
    class: Option<String>,

    /// Specifies the size, in points (1/72 in), of the media’s edge that represents
    /// the scanline direction. If this key is absent, the scanline direction is
    /// assumed to be along the X axis (of Dimensions parameter). Use of
    /// LeadingEdge can accommodate different media orientations while avoiding
    /// changes to the CTM of each PlacedObject when placing page content onto a
    /// Surface for a given Media::Dimensions array
    #[field("LE")]
    leading_edge: Option<f32>,

    /// When true, indicates that the device shall wait for input from an operator
    #[field("MF")]
    manual_feed: Option<bool>,

    /// A Media object. Describes the type of media to use. If Position has been
    /// specified, the value of Media indicates the type of media which should be
    /// loaded in the designated media source
    #[field("Me")]
    media: Option<Media>,

    /// In a device that has numbered input sources, identifies which source to use
    #[field("Po")]
    position: Option<i32>,
}

#[derive(Debug, Clone, FromObj)]
struct Media {
    /// A string identifying a user- or site-specific type of media, such as
    /// LetterHead, 3-hole, or Transparency
    #[field("Ct")]
    category: Option<String>,

    /// An array of four non-negative numbers [minX minY maxX maxY] describing an
    /// acceptable range of widths (between minX and maxX inclusive) and heights
    /// (between minY and maxY inclusive) for the medium, expressed in points
    /// (1/72 inch). A medium of fixed width X and fixed height Y is specified as
    /// [X Y X Y].
    ///
    /// In simple printing, the X,Y values imply the content orientation.
    ///
    /// For roll-fed media, the device shall advance the media at least the
    /// minimum value (either minX or minY) for the direction determined by
    /// MediaSource::LeadingEdge
    #[field("Dm")]
    dimensions: Option<[f32; 4]>,

    /// An arbitrary string identifying the color of the medium
    #[field("MC")]
    media_color: Option<String>,

    /// The weight of the medium, in grams per square meter
    #[field("We")]
    weight: Option<f32>,
}

#[derive(Debug, Clone, FromObj)]
struct MediaUsage {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct PrintLayout {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Rendering<'a> {
    /// A dictionary that specifies device rendering parameters
    #[field("DRI")]
    device_rendering_info: Option<Dictionary<'a>>,

    /// If true, a page image is produced which is reflected around the scanline
    /// direction of the device. This is accomplished by device-dependent means
    #[field("MP")]
    mirror_print: Option<bool>,

    /// If true, a page is produced which is the negative image of the page. This is
    /// accomplished by device-dependent means
    #[field("NP")]
    negative_print: Option<bool>,

    /// If true, any enhancements available on the device will be invoked
    #[field("Po")]
    post_rendering_enhance: Option<bool>,

    /// Describes product-specific details related to post-rendering image
    /// enhancement
    #[field("PoD")]
    post_rendering_enhance_details: Option<Dictionary<'a>>,

    /// If true, any enhancements available on the device will be invoked
    #[field("Pr")]
    pre_rendering_enhance: Option<bool>,

    /// Describes product-specific details related to pre-rendering image enhancement
    #[field("PrD")]
    pre_rendering_enhance_details: Option<Dictionary<'a>>,

    /// Resolution indicates the resolution for the physical device to apply,
    /// expressed in pixels per inch
    #[field("R")]
    resolution: Option<[f32; 2]>,

    /// A positive integer indicating the number of values each color component may
    /// have, or in the monochrome case, the number of gray levels
    #[field("V")]
    values_per_color_component: Option<i32>,
}

#[derive(Debug, Clone, FromObj)]
struct Scheduling {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct Trapping {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct TrappingParameters {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct TrapRegion {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct JTFile<'a> {
    /// Array of Audit objects. Each Audit object records some operation which
    /// affected the physical files referenced by this object’s File or
    /// FilesDictionary key
    #[field("A")]
    audit: Option<Vec<Audit<'a>>>,

    /// Array of Address objects identifying the person(s) who created the file
    #[field("Au")]
    authors: Option<Vec<Address>>,

    /// Human-readable notes regarding the file
    #[field("Cm")]
    comment: Option<String>,

    /// The name of the process or application that created the file
    #[field("CP")]
    creating_process: Option<String>,

    /// An array of dictionaries. Parameters used by the decoding filters specified
    /// with the Filters key. The number and types of items in the array must
    /// match the items in the Filters array. If a filter has no parameters,
    /// enter an empty dictionary
    #[field("DP")]
    decode_params: Option<Vec<Dictionary<'a>>>,

    #[field("Fi")]
    file: Option<FileSpec>,

    /// Keys are Colorant names and values are file specification objects
    #[field("FD")]
    files_dictionary: Option<Dictionary<'a>>,

    /// Determines whether or not this file, or set of files for preseparated files,
    /// will be retained beyond the Scheduling object’s Retain period
    #[field("FR")]
    file_retention: Option<FileRetention>,

    /// Identifies the type of data in File. Certain Job Ticket Processors may fail
    /// if FileType is not specified
    ///
    /// FileType must be a MIME file type as recorded by the Internet Assigned
    /// Numbers Authority (IANA)
    #[field("FT")]
    file_type: Option<String>,

    // todo: better type
    #[field("Fl")]
    filters: Option<Object<'a>>,

    /// This object specifies the constraints used to check the file and the results
    /// of preflight operations. It may also provide a complete inventory of the
    /// characteristics of the file
    #[field("Pf")]
    preflight: Option<Preflight>,

    /// Specifies a password to use when decrypting an encrypted file. Decryption is
    /// required before a Job Ticket Processor can process the content of an
    /// encrypted file. Note that the encryption method is not specified or
    /// controlled by the job ticket; the Job Ticket Processor is expected to
    /// know what encryption method is in use, and how to apply the password
    #[field("Pw")]
    password: Option<String>,

    /// An array of PlaneOrder objects which specifies the order of colorant planes
    /// which occur in the file referenced by the File key. Presence of this key
    /// indicates that the file referenced is pre-separated. Omission of this key
    /// indicates that the file is not pre-separated, unless the
    /// JTFile::FilesDictionary key is present
    ///
    /// In order to identify all colorant planes for a specific virtual page, the Job
    /// Ticket Processor must examine the objects in this array in sequence
    ///
    /// Note that the colorant names in this dictionary are subject to colorant
    /// aliasing as specified in the JobTicketContents object for the job
    #[field("PO")]
    plane_order: Option<Vec<PlaneOrder>>,

    /// The date and time the file was last changed
    #[field("RD")]
    revision_date: Option<Date>,

    /// Indicates whether the file has been trapped
    ///
    /// When this key is True, a Job Ticket Processor shall not trap the referenced
    /// file unless the processor can identify the existing traps and remove or
    /// update them
    ///
    /// When the FileType is application/pdf, this key is defined in accordance with
    /// the PDF 1.3 language specification, and it is an error if the value of
    /// this key is inconsistent with the value of the Trapped key in the file
    #[field("TR")]
    trapped: Option<Trapped>,
}

#[pdf_enum]
enum FileRetention {
    /// Retain this file until the job successfully completes and the Scheduling
    /// object’s Retain period expires
    UntilSuccess = "UntilSuccess",

    /// Never retain this file after the Scheduling object’s Retain period expires,
    /// even if the job failed
    Never = "Never",

    /// Retain this file, regardless of when the job completes
    Always = "Always",
}

#[pdf_enum]
enum FileSpec {
    /// The document is contained in the same PDF as this job ticket
    This = "This",

    /// The document will follow the job ticket when the two are streamed separately
    /// to a device
    Follows = "Follows",
}

#[derive(Debug, Clone, FromObj)]
struct PageRange<'a> {
    /// A rectangle in page coordinate space units specifying a region of the page.
    /// The BleedBox encompasses all marks which are intended to be imaged on a
    /// final trimmed page or spread, including content which may extend outside
    /// the boundaries of the trimmed page or spread. The BleedBox represents the
    /// maximum extent of the final trimmed page output in a production
    /// environment. In such environments, a “bleed area” is desired, to
    /// accommodate physical limitations of cutting, folding and trimming
    /// equipment. The BleedBox shall not extend outside the MediaBox, and if it
    /// does, the effective BleedBox shall be the intersection of the BleedBox
    /// with the MediaBox
    ///
    /// When absent, the value of MediaBox is used for the BleedBox
    #[field("Bl")]
    bleed_box: Option<Rectangle>,

    /// A ColorantControl object describes how to control output color rendering
    #[field("Co")]
    colorant_control: Option<ColorantControl>,

    /// A non-negative integer specifying how many copies of this PageRange are
    /// produced for each copy of the Document object
    ///
    /// This value multiplied by the value of Copies for the Document object this
    /// determines the total number of times the PageRange appears in the job
    ///
    /// This value is ignored when the job is printed using a Layout; it applies only
    /// to simple printing, or when printing a PrintLayout
    #[field("Cp")]
    copies: Option<i32>,

    /// A single instance of blank or alternate page content may be inserted to
    /// ensure that the PageRange starts on the designated page position (odd or
    /// even). This object will specify the alternate page content explicitly.
    /// This object is applied after NewSheet is satisfied. Applies only to
    /// PrintLayout printing
    #[field("IP")]
    insert_page: Option<InsertPage>,

    /// Inserts a sheet after each copy of the job
    #[field("IS")]
    insert_sheet: Option<InsertSheet>,

    /// When an integer, JTFile is an index into the Files array of the Document
    /// object parent to this PageRange object. Index values begin with zero
    /// (first element in Files array)
    ///
    /// When a dictionary, JTFile is a JTFile object
    // todo: type
    #[field("JTF")]
    jt_file: Object<'a>,

    /// A rectangle in page coordinate space units specifying a region that contains
    /// the maximum imageable area of the page. This rectangle includes any
    /// extended area surrounding the finished page for bleed, printers marks, or
    /// other similar purpose. Content outside the MediaBox may be safely
    /// discarded without changing the meaning of the PDL file
    ///
    /// The value specified by this key overrides any similar information provided
    /// within the PDL file (such as a MediaBox key in a PDF file).
    ///
    /// If absent, the value of Document::MediaBox is used
    #[field("MB")]
    media_box: Option<Rectangle>,

    /// Specifies a completion of the current sheet, if one is being imaged, in order
    /// to have this set of page content begin with a new sheet of media. This
    /// object may also specify a sheet to be inserted as the first sheet of the
    /// PageRange, and if so, whether the inserted sheet will be imaged or blank.
    /// If NewSheet is present, each copy of this PageRange will include this
    /// insert. Applies only to PrintLayout or simple printing
    #[field("Ns")]
    new_sheet: Option<InsertSheet>,

    /// Settings in this dictionary take precedence over settings in the Document
    /// object or JobTicketContents object’s Rendering dictionary
    #[field("R")]
    rendering: Option<Rendering<'a>>,

    /// Settings in this dictionary take precedence over settings in the Document
    /// object’s or JobTicketContents object’s Trapping dictionary
    #[field("T")]
    trapping: Option<Trapping>,

    /// A rectangle in page coordinate space units specifying a region which is the
    /// intended finished (trimmed) size of the page. For example, the dimensions
    /// of an A4 sheet of paper. The TrimBox shall not extent outside the
    /// MediaBox and if it does, the effective TrimBox shall be the intersection
    /// of the TrimBox with the MediaBox. In some cases, the MediaBox may be
    /// larger than the TrimBox and include printing instructions, color bars,
    /// cut marks, or other printers marks
    ///
    /// When absent, the value of MediaBox is used for the TrimBox
    #[field("TB")]
    trim_box: Option<Rectangle>,

    /// Specifies if a sheet will be inserted at the end of this PageRange. This
    /// object will specify how to complete the current Sheet being imaged. The
    /// inserted sheet may be blank or imaged with page content specified in this
    /// object. If Trailer is present, each copy of this PageRange will include
    /// this insert. Applies only to PrintLayout or simple printing
    #[field("Tl")]
    trailer: Option<InsertSheet>,

    /// A descriptive name to apply to the trap network which will be produced by a
    /// trapping application as a result of the TrapRegion objects for this
    /// PageRange
    #[field("TD")]
    trapping_description: Option<String>,

    /// A dictionary in which each key is the name of a TrappingParameter set and
    /// each value is a TrappingParameters object. These objects specify the sets
    /// of trapping parameters which will be used to create trap networks for
    /// pages in this PageRange
    ///
    /// If absent, TrappingParameters objects specified by the
    /// Document::TrappingParameters key are used
    #[field("TP")]
    trapping_parameters: Option<TrappingParameters>,

    /// An array of TrapRegion objects. These objects specify the trapping regions
    /// and trapping parameters which will be used to create trap networks for
    /// pages in this PageRange
    ///
    /// Note that trap networks are created for a page only when there is at least
    /// one TrapRegion object for that page
    #[field("TR")]
    trap_regions: Option<Vec<TrapRegion>>,

    /// An array of two integers [N M] where N and M are page numbers in the
    /// specified file and M > N. If omitted, all pages of the JTFile object are
    /// used
    ///
    /// The first page in a file is always page 0. To specify a range of one page,
    /// use N=M. However, the value of –1 for M is treated as a special case (see
    /// below).
    ///
    /// When `M` is -1, this indicates the last page in a file. This is useful for
    /// printing ranges of pages when the number of pages in a file is unknown.
    /// For example, /Which [3 –1] instructs the Job Ticket Processor to print
    /// pages 4 through the end of the file
    ///
    /// An error occurs if this array specifies pages which fall outside the range of
    /// total pages in the file
    ///
    /// Note that the case of M < N, which was allowed in PJTF 1.0 to indicated
    /// printing in inverse order, is eliminated for PJTF 1.1
    #[field("W")]
    which: Option<[i32; 2]>,
}

#[derive(Debug, Clone, FromObj)]
struct Preflight {
    // todo:
}

#[derive(Debug, Clone, FromObj)]
struct PlaneOrder {
    // todo:
}

#[pdf_enum]
enum TrappingSourceSelector {
    /// Only TrappingParameters and TrapRegion objects in JobTicketContents, Document
    /// and PageRange objects are used. All others are ignored
    Contents = "Contents",

    /// Only Trapping and TrappingParameters objects in Layout objects and TrapRegion
    /// objects in PlacedObject objects are used. All others are ignored
    Layout = "Layout",

    /// Only Trapping and TrappingParameters objects in PrintLayout objects and
    /// TrapRegion objects in PlacedObject objects are used. All others are
    /// ignored.
    PrintLayout = "PrintLayout",

    /// All trapping information is ignored and no trap networks are created
    None = "None",
}
