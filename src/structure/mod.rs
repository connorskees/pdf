use crate::{
    assert_empty,
    data_structures::{NameTree, NumberTree},
    error::{ParseError, PdfResult},
    objects::{Dictionary, Object, ObjectType, Reference},
    pdf_enum, Resolve,
};

#[derive(Debug)]
pub struct StructTreeRoot<'a> {
    /// The immediate child or children of the structure tree root in
    /// the structure hierarchy. The value may be either a dictionary
    /// representing a single structure element or an array of such
    /// dictionaries.
    k: Option<Vec<StructureElement<'a>>>,

    /// A name tree that maps element identifiers to the structure elements
    /// they denote.
    ///
    /// Required if any structure elements have element identifiers
    id_tree: Option<NameTree>,

    /// A number tree used in finding the structure elements to which content
    /// items belong. Each integer key in the number tree shall correspond to
    /// a single page of the document or to an individual object (such as an
    /// annotation or an XObject) that is a content item in its own right.
    ///
    /// The integer key shall be the value of the StructParent or StructParents
    /// entry in that object. The form of the associated value shall depend on
    /// the nature of the object: For an object that is a content item in its own
    /// right, the value shall be an indirect reference to the object's parent
    /// element (the structure element that contains it as a content item).
    ///
    /// For a page object or content stream containing marked-content sequences that
    /// are content items, the value shall be an array of references to the parent
    /// elements of those marked-content sequences.
    ///
    /// Required if any structure element contains content items
    parent_tree: Option<NumberTree<'a>>,

    /// An integer greater than any key in the parent tree, shall be used as a
    /// key for the next entry added to the tree.
    parent_tree_next_key: Option<i32>,

    /// A dictionary that shall map the names of structure types used in the document
    /// to their approximate equivalents in the set of standard structure types
    role_map: Option<Dictionary<'a>>,

    /// A dictionary that shall map name objects designating attribute classes to the
    /// corresponding attribute objects or arrays of attribute objects
    class_map: Option<Dictionary<'a>>,
}

impl<'a> StructTreeRoot<'a> {
    const TYPE: &'static str = "StructTreeRoot";

    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, true)?;

        let k = dict
            .get_object("K", resolver)?
            .map(|obj| StructureElement::from_obj(obj, resolver))
            .transpose()?;

        let id_tree = dict
            .get_dict("IdTree", resolver)?
            .map(|dict| NameTree::from_dict(dict, resolver))
            .transpose()?;
        let parent_tree = dict
            .get_dict("ParentTree", resolver)?
            .map(|dict| NumberTree::from_dict(dict, resolver))
            .transpose()?;
        let parent_tree_next_key = dict.get_integer("ParentTreeNextKey", resolver)?;
        let role_map = dict.get_dict("RoleMap", resolver)?;
        let class_map = dict.get_dict("ClassMap", resolver)?;

        assert_empty(dict);

        Ok(Self {
            k,
            id_tree,
            parent_tree,
            parent_tree_next_key,
            role_map,
            class_map,
        })
    }
}

#[derive(Debug)]
struct StructureElement<'a> {
    /// The structure type, a name object identifying the nature of the structure
    /// element and its role within the document, such as a chapter, paragraph, or footnote
    s: StructureType,

    /// The structure element that is the immediate parent of this one in the structure hierarchy
    p: Reference,

    /// The element identifier, a byte string designating this structure element. The string
    /// shall be unique among all elements in the document's structure hierarchy. The IDTree
    /// entry in the structure tree root defines the correspondence between element identifiers
    /// and the structure elements they denote
    // todo: byte string
    id: Option<String>,

    /// A page object representing a page on which some or all of the content items designated
    /// by the K entry shall be rendered
    pg: Option<Reference>,

    /// The children of this structure element. The value of this entry may be one of the following
    /// objects or an array consisting of one or more of the following objects:
    ///
    ///   * A structure element dictionary denoting another structure element
    ///   * An integer marked-content identifier denoting a marked-content sequence
    ///   * A marked-content reference dictionary denoting a marked-content sequence
    ///   * An object reference dictionary denoting a PDF object
    ///
    /// Each of these objects other than the first (structure element dictionary) shall be considered
    /// to be a content item. If the value of K is a dictionary containing no Type entry, it shall be
    /// assumed to be a structure element dictionary.
    k: Option<Vec<StructureElementChild<'a>>>,

    /// A single attribute object or array of attribute objects associated with this structure
    /// element. Each attribute object shall be either a dictionary or a stream. If the value of
    /// this entry is an array, each attribute object in the array may be followed by an integer
    /// representing its revision number
    // todo: what type is this
    a: Option<Object<'a>>,

    /// An attribute class name or array of class names associated with this structure element.
    ///
    /// If the value of this entry is an array, each class name in the array may be followed by an
    /// integer representing its revision number.
    ///
    /// If both the A and C entries are present and a given attribute is specified by both, the one
    /// specified by the A entry shall take precedence
    // todo: what type is this
    c: Option<Object<'a>>,

    /// The current revision number of this structure element. The value shall be a non-negative
    /// integer.
    ///
    /// Default value: 0
    r: u32,

    /// The title of the structure element, a text string representing it in human-readable form. The
    /// title should characterize the specific structure element, such as Chapter 1, rather than merely
    /// a generic element type, such as Chapter.
    t: Option<String>,

    /// A language identifier specifying the natural language for all text in the structure element
    /// except where overridden by language specifications for nested structure elements or marked content.
    /// If this entry is absent, the language (if any) specified in the document catalogue applies
    lang: Option<String>,

    /// An alternate description of the structure element and its children in human-readable form,
    /// which is useful when extracting the document's contents in support of accessibility to users
    /// with disabilities or for other purposes
    alt: Option<String>,

    /// The expanded form of an abbreviation
    e: Option<String>,

    /// Text that is an exact replacement for the structure element and its children. This replacement
    /// text (which should apply to as small a piece of content as possible) is useful when extracting
    /// the document's contents in support of accessibility to users with disabilities or for other purposes
    actual_text: Option<String>,
}

impl<'a> StructureElement<'a> {
    const TYPE: &'static str = "StructElem";

    pub fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Vec<Self>> {
        Ok(match resolver.resolve(obj)? {
            Object::Array(arr) => arr
                .into_iter()
                .map(|obj| StructureElement::from_dict(resolver.assert_dict(obj)?, resolver))
                .collect::<PdfResult<Vec<StructureElement>>>()?,
            Object::Dictionary(dict) => vec![StructureElement::from_dict(dict, resolver)?],
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Array, ObjectType::Dictionary],
                    // found,
                });
            }
        })
    }

    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, false)?;

        let s = StructureType::from_str(dict.expect_name("S", resolver)?);
        let p = dict.expect_reference("P")?;
        let id = dict.get_string("ID", resolver)?;
        let pg = dict.get_reference("Pg")?;

        let k = dict
            .get_object("K", resolver)?
            .map(|obj| StructureElementChild::from_obj(obj, resolver))
            .transpose()?;

        let a = None;
        let c = None;

        let r = dict.get_unsigned_integer("R", resolver)?.unwrap_or(0);
        let t = dict.get_string("T", resolver)?;
        let lang = dict.get_string("Lang", resolver)?;
        let alt = dict.get_string("Alt", resolver)?;
        let e = dict.get_string("E", resolver)?;
        let actual_text = dict.get_string("ActualText", resolver)?;

        assert_empty(dict);

        Ok(Self {
            s,
            p,
            id,
            pg,
            k,
            a,
            c,
            r,
            t,
            lang,
            alt,
            e,
            actual_text,
        })
    }
}

#[derive(Debug)]
enum StructureElementChild<'a> {
    StructureElement(Box<StructureElement<'a>>),
    ObjectReferenceDictionary(ObjectReferenceDictionary),
    MarkedContentIdentifier(i32),
    MarkedContentReferenceDictionary(MarkedContentReferenceDictionary),
}

impl<'a> StructureElementChild<'a> {
    pub fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Vec<Self>> {
        Ok(match resolver.resolve(obj)? {
            Object::Integer(identifier) => {
                vec![StructureElementChild::MarkedContentIdentifier(identifier)]
            }
            Object::Dictionary(dict) => vec![Self::from_dict(dict, resolver)?],
            Object::Array(arr) => arr
                .into_iter()
                .map(|obj| Self::from_obj(obj, resolver))
                .try_fold(Vec::new(), |mut init, next| -> PdfResult<Vec<Self>> {
                    init.append(&mut next?);

                    Ok(init)
                })?,
            found => {
                return Err(ParseError::MismatchedObjectTypeAny {
                    expected: &[
                        ObjectType::Array,
                        ObjectType::Dictionary,
                        ObjectType::Integer,
                    ],
                    // found,
                });
            }
        })
    }

    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let ty = dict.get_name("Type", resolver)?;

        Ok(match ty.as_deref() {
            None => Self::StructureElement(Box::new(StructureElement::from_dict(dict, resolver)?)),
            Some(ObjectReferenceDictionary::TYPE) => {
                Self::ObjectReferenceDictionary(ObjectReferenceDictionary::from_dict(dict)?)
            }
            Some(MarkedContentReferenceDictionary::TYPE) => Self::MarkedContentReferenceDictionary(
                MarkedContentReferenceDictionary::from_dict(dict, resolver)?,
            ),
            Some(StructureElement::TYPE) => {
                Self::StructureElement(Box::new(StructureElement::from_dict(dict, resolver)?))
            }
            Some(v) => todo!("{:?}", v),
        })
    }
}

#[derive(Debug)]
struct ObjectReferenceDictionary {
    /// The page object of the page on which the object shall be rendered. This entry
    /// overrides any Pg entry in the structure element containing the object reference;
    /// it shall be used if the structure element has no such entry.
    pg: Option<Reference>,

    /// The referenced object
    obj: Reference,
}

impl ObjectReferenceDictionary {
    const TYPE: &'static str = "OBJR";

    pub fn from_dict(mut dict: Dictionary) -> PdfResult<Self> {
        let pg = dict.get_reference("Pg")?;
        let obj = dict.expect_reference("Obj")?;

        Ok(Self { pg, obj })
    }
}

#[derive(Debug)]
struct MarkedContentReferenceDictionary {
    /// The page object representing the page on which the graphics objects in the marked-content
    /// sequence shall be rendered. This entry overrides any Pg entry in the structure element
    /// containing the marked-content reference; it shall be required if the structure element
    /// has no such entry.
    pg: Option<Reference>,

    /// The content stream containing the marked-content sequence. This entry should be present
    /// only if the marked-content sequence resides in a content stream other than the content
    /// stream for the page. If this entry is absent, the marked-content sequence shall be contained
    /// in the content stream of the page identified by Pg (either in the markedcontent reference
    /// dictionary or in the parent structure element)
    stm: Option<Reference>,

    /// The PDF object owning the stream identified by Stems annotation to which an appearance
    /// stream belongs.
    stm_own: Option<Reference>,

    /// The marked-content identifier of the marked-content sequence within its content stream.
    mcid: i32,
}

impl MarkedContentReferenceDictionary {
    const TYPE: &'static str = "MCR";

    pub fn from_dict<'a>(
        mut dict: Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let pg = dict.get_reference("Pg")?;
        let stm = dict.get_reference("Stm")?;
        let stm_own = dict.get_reference("StmOwn")?;
        let mcid = dict.expect_integer("MCID", resolver)?;

        Ok(Self {
            pg,
            stm,
            stm_own,
            mcid,
        })
    }
}

#[derive(Debug)]
enum StructureType {
    Standard(StandardStructureType),
    Other(String),
}

impl StructureType {
    pub fn from_str(s: String) -> Self {
        if let Ok(standard) = StandardStructureType::from_str(&s) {
            StructureType::Standard(standard)
        } else {
            StructureType::Other(s)
        }
    }
}

pdf_enum!(
    #[derive(Debug)]
    enum StandardStructureType {
        /// A complete document. This is the root element of any structure tree containing
        /// multiple parts or multiple articles
        Document = "Document",

        /// A large-scale division of a document. This type of element is appropriate for
        /// grouping articles or sections.
        Part = "Part",

        /// A relatively self-contained body of text constituting a single narrative or
        /// exposition. Articles should be disjoint; that is, they should not contain other
        /// articles as constituent elements
        Article = "Art",

        /// A container for grouping related content elements
        ///
        /// For example, a section might contain a heading, several introductory paragraphs,
        /// and two or more other sections nested within it as subsections.
        Section = "Sect",

        /// A generic block-level element or group of elements
        Division = "Div",

        /// A portion of text consisting of one or more paragraphs attributed to someone
        /// other than the author of the surrounding text.
        BlockQuote = "BlockQuote",

        /// A brief portion of text describing a table or figure
        Caption = "Caption",

        /// A list made up of table of contents item entries (structure type TOCI) and/or
        /// other nested table of contents entries (TOC)
        ///
        /// A TOC entry that includes only TOCI entries represents a flat hierarchy. A TOC
        /// entry that includes other nested TOC entries (and possibly TOCI entries) represents
        /// a more complex hierarchy. Ideally, the hierarchy of a top level TOC entry reflects
        /// the structure of the main body of the document.
        TableOfContents = "TOC",

        /// An individual member of a table of contents. This entry's children may be any of
        /// the following structure types:
        ///   * Lbl - A label
        ///   * Reference - A reference to the title and the page number
        ///   * NonStruct - Non-structure elements for wrapping a leader artifact
        ///   * P - Descriptive text
        ///   * TOC - Table of content elements for hierarchical tables of content, as described
        ///           for the TOC entry
        TableOfContentsItem = "TOCI",

        /// A sequence of entries containing identifying text accompanied by reference elements
        /// (structure type Reference) that point out occurrences of the specified text in the main
        /// body of a document.
        Index = "Index",

        /// A grouping element having no inherent structural significance; it serves solely for
        /// grouping purposes. This type of element differs from a division (structure type Div)
        /// in that it shall not be interpreted or exported to other document formats; however,
        /// its descendants shall be processed normally.
        NonStructuralElement = "NonStruct",

        /// A grouping element containing private content belonging to the application producing it.
        /// The structural significance of this type of element is unspecified and shall be determined
        /// entirely by the conforming writer. Neither the Private element nor any of its descendants
        /// shall be interpreted or exported to other document formats.
        Private = "Private",
        // todo: rest of std structure types
    }
);
