use std::{borrow::Cow, convert::TryFrom};

use crate::{
    catalog::assert_len,
    error::PdfResult,
    objects::{Dictionary, Object},
    stream::StreamDict,
    trailer::Trailer,
    Resolve,
};

pub(super) mod parser;

#[derive(Debug)]
pub struct XrefStream<'a> {
    pub(crate) dict: XrefStreamDict<'a>,
    pub(crate) stream: Cow<'a, [u8]>,
}

#[derive(Debug)]
/// Values in this dictionary must *not* be indirect references
pub struct XrefStreamDict<'a> {
    pub(crate) stream_dict: StreamDict<'a>,
    pub(crate) trailer: Trailer<'a>,

    /// An array containing a pair of integers for each subsection in this
    /// section. The first integer shall be the first object number in the
    /// subsection; the second integer shall be the number of entries in the
    /// subsection. The array shall be sorted in ascending order by object number.
    /// Subsections cannot overlap; an object number may have at most one entry
    /// in a section.
    ///
    /// Default value: [0 Size].
    pub(crate) index: Vec<(usize, usize)>,

    /// An array of integers representing the size of the fields in a single
    /// cross-reference entry. Table 18 describes the types of entries and their
    /// fields. For PDF 1.5, W always contains three integers; the value of
    /// each integer shall be the number of bytes (in the decoded stream) of
    /// the corresponding field.
    ///
    /// ### EXAMPLE
    ///
    /// [1 2 1] means that the fields are one byte, two bytes, and one byte,
    /// respectively. A value of zero for an element in the W array indicates
    /// that the corresponding field shall not be present in the stream, and the
    /// default value shall be used, if there is one. If the first element is
    /// zero, the type field shall not be present, and shall default to type 1.
    /// The sum of the items shall be the total length of each entry; it can be
    /// used with the Index array to determine the starting position of each
    /// subsection. Different cross-reference streams in a PDF file may use
    /// different values for W.
    pub(crate) w: XrefStreamFieldWidths,
}

impl<'a> XrefStreamDict<'a> {
    const TYPE: &'static str = "XRef";

    pub fn from_dict(
        mut dict: Dictionary<'a>,
        is_previous: bool,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        dict.expect_type(Self::TYPE, resolver, false)?;

        let trailer = Trailer::from_dict_ref(&mut dict, is_previous, resolver)?;
        let index = dict
            .get_arr("Index", resolver)?
            .map(|index| {
                index
                    .chunks_exact(2)
                    .map(|obj| {
                        let obj_number = obj[0].clone();
                        let num_of_entries = obj[1].clone();

                        Ok((
                            usize::try_from(resolver.assert_unsigned_integer(obj_number)?)?,
                            usize::try_from(resolver.assert_unsigned_integer(num_of_entries)?)?,
                        ))
                    })
                    .collect::<PdfResult<Vec<(usize, usize)>>>()
            })
            .transpose()?
            .unwrap_or_else(|| vec![(0, trailer.size)]);

        let w = XrefStreamFieldWidths::from_arr(dict.expect_arr("W", resolver)?, resolver)?;

        let stream_dict = StreamDict::from_dict(dict, resolver)?;

        Ok(XrefStreamDict {
            stream_dict,
            index,
            w,
            trailer,
        })
    }
}

#[derive(Debug)]
pub(crate) enum XrefStreamField {
    One = 0,
    Two = 1,
    Three = 2,
}

#[derive(Debug)]
// todo: convert to XrefStreamFieldWidths([u32; 3]) once we support derive on
// tuple structs
pub(crate) struct XrefStreamFieldWidths {
    field0: u32,
    field1: u32,
    field2: u32,
}

impl XrefStreamFieldWidths {
    pub fn from_arr<'a>(
        mut arr: Vec<Object<'a>>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        assert_len(&arr, 3)?;

        let field2 = resolver.assert_unsigned_integer(arr.pop().unwrap())?;
        let field1 = resolver.assert_unsigned_integer(arr.pop().unwrap())?;
        let field0 = resolver.assert_unsigned_integer(arr.pop().unwrap())?;

        Ok(XrefStreamFieldWidths {
            field0,
            field1,
            field2,
        })
    }

    pub fn field_width(&self, field: XrefStreamField) -> usize {
        (match field {
            XrefStreamField::One => self.field0,
            XrefStreamField::Two => self.field1,
            XrefStreamField::Three => self.field2,
        }) as usize
    }

    pub fn total_width(&self) -> u32 {
        self.field0 + self.field1 + self.field2
    }
}
