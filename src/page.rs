use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    catalog::{
        AdditionalActions, BoxColorInfo, GroupAttributes, MetadataStream, NavigationNode,
        PagePiece, SeparationInfo, Transitions, Viewport,
    },
    data_structures::Rectangle,
    date::Date,
    error::PdfResult,
    objects::{Dictionary, TypeOrArray},
    pdf_enum,
    resources::Resources,
    stream::Stream,
    Reference, Resolve,
};

pub struct PageTree {
    pub kids: Vec<PageNode>,
    pub pages: HashMap<Reference, PageNode>,
    pub count: usize,

    /// Fields inheritable by child nodes
    pub(crate) inheritable_page_fields: InheritablePageFields,
}

#[derive(Clone)]
pub enum PageNode {
    Root(Rc<RefCell<PageTree>>),
    Node(Rc<RefCell<PageTreeNode>>),
    Leaf(Rc<PageObject>),
}

impl PageNode {
    pub fn leaves(&self) -> Vec<Rc<PageObject>> {
        let mut leaves = Vec::new();

        match self {
            PageNode::Root(root) => {
                for kid in &root.borrow().kids {
                    leaves.append(&mut kid.leaves());
                }
            }
            PageNode::Node(node) => leaves.append(&mut node.borrow().leaves()),
            PageNode::Leaf(leaf) => leaves.push(Rc::clone(leaf)),
        }

        leaves
    }

    pub fn crop_box(&self) -> Option<Rectangle> {
        match self {
            Self::Root(tree) => tree.borrow().inheritable_page_fields.crop_box,
            Self::Node(node) => node
                .borrow()
                .inheritable_page_fields
                .crop_box
                .or_else(|| node.borrow().parent.crop_box()),
            Self::Leaf(leaf) => leaf.crop_box(),
        }
    }
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
                .field("kids", &r.borrow().kids)
                .field("count", &r.borrow().count)
                .finish(),
            Self::Leaf(r) => write!(f, "{:#?}", r),
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

    /// Fields inheritable by child nodes
    pub(crate) inheritable_page_fields: InheritablePageFields,
}

impl PageTreeNode {
    pub fn leaves(&self) -> Vec<Rc<PageObject>> {
        let mut leaves = Vec::new();

        for node in &self.kids {
            match node {
                PageNode::Root(..) => unreachable!(),
                PageNode::Leaf(leaf) => leaves.push(Rc::clone(leaf)),
                PageNode::Node(node) => leaves.append(&mut node.borrow().leaves()),
            }
        }

        leaves
    }
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
    ///
    /// Inheritable
    pub resources: Option<Resources>,

    /// A rectangle, expressed in default user space units, that shall define
    /// the boundaries of the physical medium on which the page shall be displayed
    /// or printed.
    ///
    /// Inheritable
    pub media_box: Option<Rectangle>,

    /// A rectangle, expressed in default user space units, that shall
    /// define the visible region of default user space. When the page
    /// is displayed or printed, its contents shall be clipped (cropped)
    /// to this rectangle and then shall be imposed on the output medium
    /// in some implementation-defined manner.
    ///
    /// Default value: the value of `media_box`.
    ///
    /// Inheritable
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
    pub contents: Option<TypeOrArray<Stream>>,

    /// The number of degrees by which the page shall be rotated clockwise
    /// when displayed or printed. The value shall be a multiple of 90.
    ///
    /// Default value: 0.
    ///
    /// Inheritable
    pub rotate: Option<i32>,

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
    pub annots: Option<Vec<Reference>>,

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

impl PageObject {
    pub fn crop_box(&self) -> Option<Rectangle> {
        self.crop_box
            .or_else(|| self.parent.crop_box())
            .or(self.media_box)
    }
}

impl fmt::Debug for PageObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageNode::Leaf")
            .field("resources", &self.resources)
            .field("contents", &self.contents)
            .field("media_box", &self.media_box)
            .field("crop_box", &self.crop_box)
            .field("bleed_box", &self.bleed_box)
            .field("trim_box", &self.trim_box)
            .field("art_box", &self.art_box)
            .field("group", &self.group)
            .field("annots", &self.annots)
            .finish()
    }
}

pdf_enum!(
    #[derive(Debug)]
    pub enum TabOrder {
        Row = "R",
        Column = "C",
        Structure = "S",
    }
);

#[derive(Debug)]
pub(crate) struct InheritablePageFields {
    resources: Option<Resources>,
    media_box: Option<Rectangle>,
    crop_box: Option<Rectangle>,
    rotate: Option<i32>,
}

impl InheritablePageFields {
    pub fn new() -> Self {
        Self {
            resources: None,
            media_box: None,
            crop_box: None,
            rotate: None,
        }
    }

    pub fn from_dict(dict: &mut Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let resources = dict
            .get_dict("Resources", resolver)?
            .map(|dict| Resources::from_dict(dict, resolver))
            .transpose()?;

        let media_box = dict.get_rectangle("MediaBox", resolver)?;
        let crop_box = dict.get_rectangle("CropBox", resolver)?;

        let rotate = dict.get_integer("Rotate", resolver)?;

        Ok(Self {
            resources,
            media_box,
            crop_box,
            rotate,
        })
    }
}
