use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
};

use crate::data_structures::Matrix;

use super::{operator::PostscriptOperator, PostScriptError, PostScriptResult};

pub(super) type Name = PostScriptString;

#[derive(Debug, Clone, PartialEq)]
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
    Operator(PostscriptOperator),
}

impl PostScriptObject {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }

    pub fn into_int(self) -> PostScriptResult<i32> {
        match self {
            PostScriptObject::Int(n) => Ok(n),
            PostScriptObject::Float(f) => Ok(f.round() as i32),
            _ => anyhow::bail!(PostScriptError::TypeCheck),
        }
    }

    pub fn into_float(self) -> PostScriptResult<f32> {
        match self {
            PostScriptObject::Int(n) => Ok(n as f32),
            PostScriptObject::Float(f) => Ok(f),
            _ => anyhow::bail!(PostScriptError::TypeCheck),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Access {
    /// Normally, objects have unlimited access: all operations defined for that
    /// object are allowed. However, packed array objects always have read-only
    /// (or even more restricted) access
    #[default]
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

#[derive(Debug, Clone)]
pub(super) struct PostScriptDictionary {
    inner: HashMap<Name, PostScriptObject>,
    access: Access,
    capacity: usize,
}

impl PostScriptDictionary {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            access: Access::default(),
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
            access: Access::default(),
            capacity,
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

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl PostScriptDictionary {
    pub fn expect_dict(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<DictionaryIndex> {
        self.get_dict(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_dict(&self, key: &'static [u8]) -> PostScriptResult<Option<DictionaryIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Dictionary(dict)) => Ok(Some(*dict)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_name(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<PostScriptString> {
        self.get_name(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_name(&self, key: &'static [u8]) -> PostScriptResult<Option<PostScriptString>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Name(s)) => Ok(Some(s.clone())),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_str(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<StringIndex> {
        self.get_str(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_str(&self, key: &'static [u8]) -> PostScriptResult<Option<StringIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::String(s)) => Ok(Some(*s)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_array(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<ArrayIndex> {
        self.get_array(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_array(&self, key: &'static [u8]) -> PostScriptResult<Option<ArrayIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Array(a)) => Ok(Some(*a)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_number(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<f32> {
        self.get_number(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_number(&self, key: &'static [u8]) -> PostScriptResult<Option<f32>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Float(n)) => Ok(Some(*n)),
            Some(PostScriptObject::Int(n)) => Ok(Some(*n as f32)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_integer(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<i32> {
        self.get_integer(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_integer(&self, key: &'static [u8]) -> PostScriptResult<Option<i32>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Int(n)) => Ok(Some(*n)),
            Some(PostScriptObject::Float(f)) => Ok(Some(f.round() as i32)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_procedure(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<ArrayIndex> {
        self.get_procedure(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_procedure(&self, key: &'static [u8]) -> PostScriptResult<Option<ArrayIndex>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Array(n)) => Ok(Some(*n)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
        }
    }

    pub fn expect_bool(
        &self,
        key: &'static [u8],
        error: PostScriptError,
    ) -> PostScriptResult<bool> {
        self.get_bool(key)?.ok_or(anyhow::anyhow!(error))
    }

    pub fn get_bool(&self, key: &'static [u8]) -> PostScriptResult<Option<bool>> {
        match self.inner.get(&Name::from_bytes(key.to_vec())) {
            Some(PostScriptObject::Bool(b)) => Ok(Some(*b)),
            Some(..) => anyhow::bail!(PostScriptError::TypeCheck),
            None => Ok(None),
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
            .ok_or(anyhow::anyhow!(PostScriptError::RangeCheck))
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

    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
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
        write!(f, "{:?}", String::from_utf8_lossy(&self.inner))?;

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

    pub fn new_procedure(inner: Vec<PostScriptObject>) -> Self {
        Self {
            capacity: 0,
            inner,
            access: Access::ExecuteOnly,
        }
    }

    pub fn put(&mut self, idx: usize, obj: PostScriptObject) {
        self.inner[idx] = obj;
    }

    pub fn get(&self, idx: usize) -> PostScriptResult<&PostScriptObject> {
        self.inner
            .get(idx)
            .ok_or(anyhow::anyhow!(PostScriptError::RangeCheck))
    }

    pub(super) fn set_access(&mut self, access: Access) {
        self.access = access;
    }

    pub(super) fn access(&self) -> Access {
        self.access
    }

    pub(super) fn into_inner(self) -> Vec<PostScriptObject> {
        self.inner
    }

    pub(super) fn as_inner(&self) -> &[PostScriptObject] {
        &self.inner
    }

    pub(super) fn len(&self) -> usize {
        self.inner.len()
    }

    pub(super) fn as_matrix(&self) -> PostScriptResult<Matrix> {
        fn expect_number(obj: &PostScriptObject) -> PostScriptResult<f32> {
            match obj {
                PostScriptObject::Float(n) => Ok(*n),
                PostScriptObject::Int(n) => Ok(*n as f32),
                _ => anyhow::bail!(PostScriptError::TypeCheck),
            }
        }

        if self.len() != 6 {
            println!("Invalid PostScript matrix");
            anyhow::bail!(PostScriptError::InvalidFont);
        }

        // todo: LIKELY BUG, this should be indexed in opposite order?

        let a = expect_number(&self.inner[5])?;
        let b = expect_number(&self.inner[4])?;
        let c = expect_number(&self.inner[3])?;
        let d = expect_number(&self.inner[2])?;
        let e = expect_number(&self.inner[1])?;
        let f = expect_number(&self.inner[0])?;

        Ok(Matrix::new(a, b, c, d, e, f))
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
