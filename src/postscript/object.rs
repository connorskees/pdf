use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
};

use crate::{
    data_structures::{Matrix, Rectangle},
    error::PdfResult,
    pdf_enum,
};

use super::{PostScriptError, PostScriptResult, PostscriptOperator};

type Name = PostScriptString;

#[derive(Debug, Clone)]
pub(super) enum PostScriptObject {
    Null,
    Int(i32),
    Float(f32),
    Name(Name),
    Literal(Name),
    Bool(bool),
    String(StringIndex),
    Array(ArrayIndex),
    Mark,
    File,
    Dictionary(DictionaryIndex),
    Procedure(Procedure),
    Operator(PostscriptOperator),
}

#[derive(Debug, Clone)]
pub(super) enum Access {
    /// Normally, objects have unlimited access: all operations defined for that
    /// object are allowed. However, packed array objects always have read-only
    /// (or even more restricted) access
    Unlimited,

    /// An object with read-only access may not have its value written, but may
    /// still be read or executed
    ReadOnly,

    /// An object with execute-only access may not have its value either read or
    /// written, but may still be executed by the PostScript interpreter
    ExecuteOnly,

    /// An object with no access may not be operated on in any way by a PostScript
    /// language program. Such objects are not of any direct use to PostScript language
    /// programs, but serve internal purposes that are not documented
    None,
}

impl Default for Access {
    fn default() -> Self {
        Access::Unlimited
    }
}

#[derive(Debug, Clone)]
pub(super) struct Procedure {
    pub(super) inner: Vec<PostScriptObject>,
    access: Access,
}

impl Procedure {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            access: Access::default(),
        }
    }

    pub fn from_toks(toks: Vec<PostScriptObject>) -> Self {
        Self {
            inner: toks,
            access: Access::default(),
        }
    }

    pub fn set_access(&mut self, access: Access) {
        self.access = access;
    }
}

#[derive(Debug, Clone)]
pub(super) struct PostScriptDictionary {
    inner: HashMap<Name, PostScriptObject>,
    access: Access,
}

impl PostScriptDictionary {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            access: Access::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
            access: Access::default(),
        }
    }

    pub fn insert(&mut self, key: Name, value: PostScriptObject) {
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &Name) -> Option<&PostScriptObject> {
        self.inner.get(key)
    }

    pub fn contains(&self, key: &Name) -> bool {
        self.inner.contains_key(key)
    }

    pub fn set_access(&mut self, access: Access) {
        self.access = access;
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Name, PostScriptObject)> {
        self.inner.into_iter()
    }
}

impl PostScriptDictionary {
    pub fn expect_dict(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<DictionaryIndex> {
        self.get_dict(key)?.ok_or(error)
    }

    pub fn get_dict(&self, key: &'static [u8]) -> PostScriptResult<Option<DictionaryIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Dictionary(dict)) => return Ok(Some(*dict)),
            Some(..) => return Err(PostScriptError::TypeCheck),
            None => return Ok(None),
        }
    }

    pub fn expect_str(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<StringIndex> {
        self.get_str(key)?.ok_or(error)
    }

    pub fn get_str(&self, key: &'static [u8]) -> PostScriptResult<Option<StringIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::String(s)) => return Ok(Some(*s)),
            Some(..) => return Err(PostScriptError::TypeCheck),
            None => return Ok(None),
        }
    }

    pub fn expect_number(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<f32> {
        self.get_number(key)?.ok_or(error)
    }

    pub fn get_number(&self, key: &'static [u8]) -> PostScriptResult<Option<f32>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Float(n)) => return Ok(Some(*n)),
            Some(PostScriptObject::Int(n)) => return Ok(Some(*n as f32)),
            Some(..) => return Err(PostScriptError::TypeCheck),
            None => return Ok(None),
        }
    }

    pub fn expect_bool(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<bool> {
        self.get_bool(key)?.ok_or(error)
    }

    pub fn get_bool(&self, key: &'static [u8]) -> PostScriptResult<Option<bool>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Bool(b)) => return Ok(Some(*b)),
            Some(..) => return Err(PostScriptError::TypeCheck),
            None => return Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct PostScriptString {
    inner: Vec<u8>,
    access: Access,
    len: usize,
    capacity: usize,
}

impl PostScriptString {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            len: 0,
            inner: vec![0; capacity],
            access: Access::default(),
        }
    }

    pub fn from_bytes(inner: Vec<u8>) -> Self {
        Self {
            len: inner.len(),
            capacity: inner.len(),
            inner,
            access: Access::default(),
        }
    }

    pub(super) fn set_access(&mut self, access: Access) {
        self.access = access;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn write(&mut self, bytes: &[u8]) {
        if (self.capacity - self.len) < bytes.len() {
            todo!()
        } else {
            self.inner
                .get_mut(self.len..(self.len + bytes.len()))
                .unwrap()
                .copy_from_slice(bytes);
        }
    }

    pub fn put(&mut self, idx: usize, byte: u8) {
        self.inner[idx] = byte;
    }

    pub fn get(&self, idx: usize) -> PostScriptResult<u8> {
        self.inner
            .get(idx)
            .cloned()
            .ok_or(PostScriptError::RangeCheck)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl PartialEq for PostScriptString {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Eq for PostScriptString {}

impl PartialOrd for PostScriptString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for PostScriptString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl Hash for PostScriptString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl fmt::Debug for PostScriptString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // if self.inner.is_ascii() {
        write!(f, "{:?}", String::from_utf8_lossy(&self.inner))?;
        // } else {
        //     f.debug_list().entry(&self.inner).finish()?;
        // }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct PostScriptArray {
    inner: Vec<PostScriptObject>,
    access: Access,
    capacity: usize,
}

impl PostScriptArray {
    pub fn new() -> Self {
        Self::from_objects(Vec::new())
    }

    pub fn from_objects(inner: Vec<PostScriptObject>) -> Self {
        Self {
            capacity: 0,
            inner,
            access: Access::default(),
        }
    }

    pub fn put(&mut self, idx: usize, obj: PostScriptObject) {
        self.inner[idx] = obj;
    }

    pub fn get(&self, idx: usize) -> PostScriptResult<&PostScriptObject> {
        self.inner.get(idx).ok_or(PostScriptError::RangeCheck)
    }

    pub(super) fn set_access(&mut self, access: Access) {
        self.access = access;
    }
}

pub(super) trait Increment: fmt::Debug + Eq + Hash + Copy {
    /// Initial value
    fn init() -> Self;

    /// Increments self by 1. Returns previous value
    fn increment(&mut self) -> Self;
}

macro_rules! index {
    ($name:ident) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        pub(super) struct $name(pub usize);

        impl Increment for $name {
            fn init() -> Self {
                Self(0)
            }

            fn increment(&mut self) -> Self {
                let prev = self.0;

                self.0 += 1;

                Self(prev)
            }
        }
    };
}

index!(ArrayIndex);
index!(StringIndex);
index!(DictionaryIndex);

#[derive(Debug)]
pub(super) struct Container<K: Increment, V> {
    map: HashMap<K, V>,
    counter: K,
}

// #[derive(Debug)]
// pub(super) struct StringInterner {
//     map: HashMap<StringIndex, PostScriptString>,
//     values: HashMap<PostScriptString, StringIndex>,
//     counter: StringIndex,
// }

// impl StringInterner {
//     pub fn new() -> Self {
//         Self {
//             map: HashMap::new(),
//             values: HashMap::new(),
//             counter: StringIndex::init(),
//         }
//     }

//     pub fn insert(&mut self, v: PostScriptString) -> StringIndex {
//         if let Some(idx) = self.values.get(&v) {
//             return *idx;
//         }

//         dbg!(&v);

//         let idx = self.counter.increment();

//         self.map.insert(idx, v.clone());
//         self.values.insert(v, idx);

//         idx
//     }

//     pub fn get(&self, k: &StringIndex) -> Option<&PostScriptString> {
//         self.map.get(k)
//     }

//     pub fn get_mut(&mut self, k: &StringIndex) -> Option<&mut PostScriptString> {
//         self.map.get_mut(k)
//     }
// }

impl<K: Increment, V> Container<K, V> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            counter: K::init(),
        }
    }

    pub fn insert(&mut self, v: V) -> K {
        let idx = self.counter.increment();

        self.map.insert(idx, v);

        idx
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.map.get(k)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.map.get_mut(k)
    }
}
