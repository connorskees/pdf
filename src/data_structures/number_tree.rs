use std::{collections::BTreeMap, rc::Rc};

use crate::{
    assert_empty,
    catalog::assert_len,
    error::PdfResult,
    objects::{Dictionary, Object},
    Resolve,
};

/// A number tree is similar to a name tree, except that its keys shall be integers instead of
/// strings and shall be sorted in ascending numerical order. The entries in the leaf (or root)
/// nodes containing the key-value pairs shall be named Nums instead of Names as in a name tree
#[derive(Debug)]
pub struct NumberTree<'a> {
    root: NumberTreeRoot<'a>,
}

fn get_nums<'a>(
    dict: &mut Dictionary<'a>,
    resolver: &mut dyn Resolve<'a>,
) -> PdfResult<Option<BTreeMap<i32, Object<'a>>>> {
    dict.get_arr("Nums", resolver)?
        .map(|num| {
            num.chunks_exact(2)
                .map(|nums| {
                    let num = resolver.assert_integer(nums[0].clone())?;
                    let obj = resolver.resolve(nums[1].clone())?;

                    Ok((num, obj))
                })
                .collect::<PdfResult<BTreeMap<i32, Object>>>()
        })
        .transpose()
}

impl<'a> NumberTree<'a> {
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let nums = get_nums(&mut dict, resolver)?;

        assert_empty(dict);

        Ok(Self {
            root: NumberTreeRoot { nums, kids: None },
        })
    }
}

#[derive(Debug)]
struct NumberTreeRoot<'a> {
    /// Shall be an array of indirect references to the immediate children of this
    /// node. The children may be intermediate or leaf nodes.
    ///
    /// Present iff Nums is not present
    kids: Option<Vec<Rc<NumberTreeNode<'a>>>>,

    /// Shall be an array of the form
    ///
    ///   [key1 value1 key2 value2 ... keyn valuen]
    ///
    /// where each keyi is an integer and the corresponding valuei shall be the object
    /// associated with that key
    nums: Option<BTreeMap<i32, Object<'a>>>,
}

#[derive(Debug)]
enum NumberTreeNode<'a> {
    Intermediate(NumberTreeIntermediateNode<'a>),
    Leaf(NumberTreeLeaf<'a>),
}

#[derive(Debug)]
struct NumberTreeIntermediateNode<'a> {
    kids: Vec<Rc<NumberTreeNode<'a>>>,
    limit: Limit,
}

#[derive(Debug)]
struct NumberTreeLeaf<'a> {
    nums: BTreeMap<i32, Object<'a>>,
    limit: Limit,
}

#[derive(Debug)]
struct Limit {
    max: i32,
    min: i32,
}

impl Limit {
    pub fn from_arr<'a>(mut arr: Vec<Object>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        assert_len(&arr, 2)?;

        let max = resolver.assert_integer(arr.pop().unwrap())?;
        let min = resolver.assert_integer(arr.pop().unwrap())?;

        Ok(Limit { max, min })
    }
}
