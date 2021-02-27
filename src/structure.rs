use crate::{
    error::PdfResult,
    objects::{Dictionary, TypeOrArray},
    Resolve,
};

#[derive(Debug)]
pub struct StructTreeRoot {
    /// The immediate child or children of the structure tree root in
    /// the structure hierarchy. The value may be either a dictionary
    /// representing a single structure element or an array of such
    /// dictionaries.
    k: Option<TypeOrArray<Dictionary>>,

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
    parent_tree: Option<NumberTree>,

    ///  An integer greater than any key in the parent tree, shall be used as a
    /// key for the next entry added to the tree.
    parent_tree_next_key: Option<i32>,
}

#[derive(Debug)]
struct NameTree;
#[derive(Debug)]
struct NumberTree;

impl StructTreeRoot {
    const TYPE: &'static str = "StructTreeRoot";

    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, true)?;

        todo!()
    }
}
