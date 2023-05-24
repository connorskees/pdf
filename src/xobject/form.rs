use crate::{
    catalog::{GroupAttributes, MetadataStream, PagePiece},
    data_structures::{Matrix, Rectangle},
    date::Date,
    error::PdfResult,
    optional_content::OptionalContent,
    resources::Resources,
    stream::Stream,
    Resolve,
};

use super::{reference::ReferenceXObject, OpenPrepressInterface};

#[derive(Debug, Clone)]
pub struct FormXObject<'a> {
    /// An array of four numbers in the form coordinate system (see above), giving the
    /// coordinates of the left, bottom, right, and top edges, respectively, of the form
    /// XObject's bounding box. These boundaries shall be used to clip the form XObject and
    /// to determine its size for caching
    bbox: Rectangle,

    pub stream: Stream<'a>,

    /// An array of six numbers specifying the form matrix, which maps form space into
    /// user space
    ///
    /// Default value: the identity matrix [1 0 0 1 0 0].
    matrix: Matrix,

    /// A dictionary specifying any resources (such as fonts and images) required by the
    /// form XObject.
    ///
    /// In a PDF whose version is 1.1 and earlier, all named resources used in the form
    /// XObject shall be included in the resource dictionary of each page object on which
    /// the form XObject appears, regardless of whether they also appear in the resource
    /// dictionary of the form XObject. These resources should also be specified in the
    /// form XObject's resource dictionary as well, to determine which resources are used
    /// inside the form XObject. If a resource is included in both dictionaries, it shall
    /// have the same name in both locations.
    ///
    /// In PDF 1.2 and later versions, form XObjects may be independent of the content
    /// streams in which they appear, and this is strongly recommended although not required.
    /// In an independent form XObject, the resource dictionary of the form XObject is required
    /// and shall contain all named resources used by the form XObject. These resources shall
    /// not be promoted to the outer content stream's resource dictionary, although that
    /// stream's resource dictionary refers to the form XObject
    resources: Option<Resources<'a>>,

    /// A group attributes dictionary indicating that the contents of the form XObject shall
    /// be treated as a group and specifying the attributes of that group.
    ///
    /// If a Ref entry (see below) is present, the group attributes shall also apply to
    /// the external page imported by that entry, which allows such an imported page to be
    /// treated as a group without further modification
    group: Option<GroupAttributes<'a>>,

    /// A reference dictionary identifying a page to be imported from another PDF file,
    /// and for which the form XObject serves as a proxy
    reference: Option<ReferenceXObject<'a>>,

    /// A metadata stream containing metadata for the form XObject
    metadata: Option<MetadataStream<'a>>,

    /// A page-piece dictionary associated with the form XObject
    piece_info: Option<PagePiece<'a>>,

    /// The date and time when the form XObject's contents were most recently modified. If a
    /// page-piece dictionary (PieceInfo) is present, the modification date shall be used to
    /// ascertain which of the application data dictionaries it contains correspond to the
    /// current content of the form
    last_modified: Option<Date>,

    /// The integer key of the form XObject's entry in the structural parent tree
    struct_parent: Option<i32>,

    /// The integer key of the form XObject's entry in the structural parent tree
    ///
    /// At most one of the entries StructParent or StructParents shall be present. A form XObject
    /// shall be either a content item in its entirety or a container for marked-content sequences
    /// that are content items, but not both.
    struct_parents: Option<i32>,

    /// An OPI version dictionary for the form XObject
    opi: Option<OpenPrepressInterface>,

    /// An optional content group or optional content membership dictionary specifying the
    /// optional content properties for the form XObject. Before the form is processed, its
    /// visibility shall be determined based on this entry. If it is determined to be invisible,
    /// the entire form shall be skipped, as if there were no Do operator to invoke it
    oc: Option<OptionalContent>,

    /// The name by which this form XObject is referenced in the XObject subdictionary of the
    /// current resource dictionary
    ///
    /// This entry is obsolescent and its use is no longer recommended
    name: Option<String>,
}

impl<'a> FormXObject<'a> {
    pub fn from_stream(mut stream: Stream<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let dict = &mut stream.dict.other;

        let bbox = dict.expect::<Rectangle>("BBox", resolver)?;
        let matrix = dict
            .get::<Matrix>("Matrix", resolver)?
            .unwrap_or_else(Matrix::identity);

        let resources = dict
            .get_dict("Resources", resolver)?
            .map(|dict| Resources::from_dict(dict, resolver))
            .transpose()?;

        let reference = dict.get::<ReferenceXObject>("Ref", resolver)?;
        let group = dict.get::<GroupAttributes>("Group", resolver)?;

        let metadata = dict
            .get_stream("Metadata", resolver)?
            .map(|stream| MetadataStream::from_stream(stream, resolver))
            .transpose()?;

        let piece_info = dict.get::<PagePiece>("PieceInfo", resolver)?;

        let last_modified = dict.get::<Date>("LastModified", resolver)?;
        let struct_parent = dict.get_integer("StructParent", resolver)?;
        let struct_parents = dict.get_integer("StructParents", resolver)?;
        let opi = dict
            .get_dict("OPI", resolver)?
            .map(|dict| OpenPrepressInterface::from_dict(dict, resolver))
            .transpose()?;
        let oc = dict
            .get_dict("OC", resolver)?
            .map(|dict| OptionalContent::from_dict(dict, resolver))
            .transpose()?;

        let name = dict.get_name("Name", resolver)?;

        Ok(Self {
            bbox,
            matrix,
            stream,
            resources,
            reference,
            group,
            metadata,
            piece_info,
            last_modified,
            struct_parent,
            struct_parents,
            opi,
            oc,
            name,
        })
    }
}
