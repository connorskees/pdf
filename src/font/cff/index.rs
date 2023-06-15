#[derive(Debug)]
pub struct CffIndex<'a> {
    pub count: u16,
    pub offsize: u8,
    pub offset: Vec<u32>,
    pub data: &'a [u8],
}

impl<'a> CffIndex<'a> {
    pub fn iter(&self) -> CffIndexIterator<'a, '_> {
        CffIndexIterator::new(self)
    }

    pub fn get(&self, idx: usize) -> Option<&'a [u8]> {
        let start = *self.offset.get(idx)?;
        let end = self
            .offset
            .get(idx + 1)
            .copied()
            .unwrap_or(self.data.len() as u32 - 1);

        self.data.get(start as usize - 1..end as usize - 1)
    }
}

#[derive(Debug)]
pub struct CffIndexIterator<'a, 'b> {
    index: &'b CffIndex<'a>,
    cursor: usize,
}

impl<'a, 'b> CffIndexIterator<'a, 'b> {
    pub fn new(index: &'b CffIndex<'a>) -> Self {
        Self { index, cursor: 0 }
    }
}

impl<'a, 'b> Iterator for CffIndexIterator<'a, 'b> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.index.get(self.cursor)?;
        self.cursor += 1;
        Some(n)
    }
}
