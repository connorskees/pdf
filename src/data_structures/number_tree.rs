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
pub struct NumberTree {
    root: NumberTreeRoot,
}

fn get_nums(
    dict: &mut Dictionary,
    resolver: &mut dyn Resolve,
) -> PdfResult<Option<BTreeMap<i32, Object>>> {
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

impl NumberTree {
    pub fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        let nums = get_nums(&mut dict, resolver)?;

        assert_empty(dict);

        Ok(Self {
            root: NumberTreeRoot { nums, kids: None },
        })
    }
}

#[derive(Debug)]
struct NumberTreeRoot {
    /// Shall be an array of indirect references to the immediate children of this
    /// node. The children may be intermediate or leaf nodes.
    ///
    /// Present iff Nums is not present
    kids: Option<Vec<Rc<NumberTreeNode>>>,

    /// Shall be an array of the form
    ///
    ///   [key1 value1 key2 value2 ... keyn valuen]
    ///
    /// where each keyi is an integer and the corresponding valuei shall be the object
    /// associated with that key
    nums: Option<BTreeMap<i32, Object>>,
}

#[derive(Debug)]
enum NumberTreeNode {
    Intermediate(NumberTreeIntermediateNode),
    Leaf(NumberTreeLeaf),
}

#[derive(Debug)]
struct NumberTreeIntermediateNode {
    kids: Vec<Rc<NumberTreeNode>>,
    limit: Limit,
}

#[derive(Debug)]
struct NumberTreeLeaf {
    nums: BTreeMap<i32, Object>,
    limit: Limit,
}

#[derive(Debug)]
struct Limit {
    max: i32,
    min: i32,
}

impl Limit {
    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 2)?;

        let max = resolver.assert_integer(arr.pop().unwrap())?;
        let min = resolver.assert_integer(arr.pop().unwrap())?;

        Ok(Limit { max, min })
    }
}
