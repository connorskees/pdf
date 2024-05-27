use std::{collections::BTreeMap, rc::Rc};

use crate::{
    assert_empty,
    catalog::assert_len,
    error::PdfResult,
    objects::{Dictionary, Object},
    FromObj, Resolve,
};

// todo: add docs
// todo: this is just a copy of the number tree. would be nice to fix it up a bit or
// unify the two somehow
#[derive(Debug)]
pub struct NameTree<'a> {
    root: NameTreeRoot<'a>,
}

fn get_names<'a>(
    dict: &mut Dictionary<'a>,
    resolver: &mut dyn Resolve<'a>,
) -> PdfResult<Option<BTreeMap<String, Object<'a>>>> {
    dict.get_arr("Kids", resolver)?
        .map(|names| {
            names
                .chunks_exact(2)
                .map(|names| {
                    let name = resolver.assert_string(names[0].clone())?;
                    let obj = resolver.resolve(names[1].clone())?;

                    Ok((name, obj))
                })
                .collect::<PdfResult<BTreeMap<String, Object>>>()
        })
        .transpose()
}

impl<'a> FromObj<'a> for NameTree<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut dict = resolver.assert_dict(obj)?;

        let names = get_names(&mut dict, resolver)?;

        assert_empty(dict);

        Ok(Self {
            root: NameTreeRoot { names, kids: None },
        })
    }
}

#[derive(Debug)]
struct NameTreeRoot<'a> {
    /// Shall be an array of indirect references to the immediate children of this
    /// node. The children may be intermediate or leaf nodes.
    ///
    /// Present iff Names is not present
    kids: Option<Vec<Rc<NameTreeNode<'a>>>>,

    /// Shall be an array of the form
    ///
    ///   [key1 value1 key2 value2 ... keyn valuen]
    ///
    /// where each keyi is an integer and the corresponding valuei shall be the object
    /// associated with that key
    names: Option<BTreeMap<String, Object<'a>>>,
}

#[derive(Debug)]
enum NameTreeNode<'a> {
    Intermediate(NameTreeIntermediateNode<'a>),
    Leaf(NameTreeLeaf<'a>),
}

#[derive(Debug)]
struct NameTreeIntermediateNode<'a> {
    kids: Vec<Rc<NameTreeNode<'a>>>,
    limit: Limit,
}

#[derive(Debug)]
struct NameTreeLeaf<'a> {
    names: BTreeMap<String, Object<'a>>,
    limit: Limit,
}

#[derive(Debug)]
struct Limit {
    max: String,
    min: String,
}

impl Limit {
    pub fn from_arr(mut arr: Vec<Object>, resolver: &mut dyn Resolve) -> PdfResult<Self> {
        assert_len(&arr, 2)?;

        let max = resolver.assert_string(arr.pop().unwrap())?;
        let min = resolver.assert_string(arr.pop().unwrap())?;

        Ok(Limit { max, min })
    }
}
