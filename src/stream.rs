use std::{borrow::Cow, fmt};

use crate::{
    error::{ParseError, PdfResult},
    file_specification::FileSpecification,
    filter::FilterKind,
    objects::{Dictionary, Object, ObjectType},
    Resolve,
};

#[derive(Clone, PartialEq)]
pub struct Stream<'a> {
    pub(crate) dict: StreamDict<'a>,
    pub(crate) stream: Cow<'a, [u8]>,
}

impl fmt::Debug for Stream<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stream")
            .field("dict", &self.dict)
            .field("stream", &format!("[ {} bytes ]", self.stream.len()))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DecodeParams<'a> {
    params: Vec<Option<Dictionary<'a>>>,
}

impl<'a> DecodeParams<'a> {
    pub fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let params = match resolver.resolve(obj)? {
            Object::Array(arr) => arr
                .into_iter()
                .map(|obj| match resolver.resolve(obj)? {
                    Object::Dictionary(dict) => Ok(Some(dict)),
                    Object::Null => Ok(None),
                    _ => anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                        expected: &[ObjectType::Null, ObjectType::Dictionary],
                    }),
                })
                .collect::<PdfResult<Vec<Option<Dictionary>>>>()?,
            Object::Dictionary(dict) => vec![Some(dict)],
            _ => {
                anyhow::bail!(ParseError::MismatchedObjectTypeAny {
                    expected: &[ObjectType::Array, ObjectType::Dictionary],
                });
            }
        };

        Ok(Self { params })
    }

    pub fn get(&self, idx: usize) -> Option<&Dictionary<'a>> {
        self.params.get(idx).and_then(|d| d.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamDict<'a> {
    /// The number of bytes from the beginning of the line following the keyword stream
    /// to the last byte just before the keyword endstream. (There may be an additional
    /// EOL marker, preceding endstream, that is not included in the count and is not
    /// logically part of the stream data.)
    pub len: usize,

    /// The name of a filter that shall be applied in processing the stream data found
    /// between the keywords stream and endstream, or an array of zero, one or several
    /// names. Multiple filters shall be specified in the order in which they are to be
    /// applied
    pub filter: Option<Vec<FilterKind>>,

    /// A parameter dictionary or an array of such dictionaries, used by the filters
    /// specified by Filter. If there is only one filter and that filter has parameters,
    /// DecodeParms shall be set to the filter's parameter dictionary unless all the
    /// filter's parameters have their default values, in which case the DecodeParms
    /// entry may be omitted. If there are multiple filters and any of the filters has
    /// parameters set to nondefault values, DecodeParms shall be an array with one
    /// entry for each filter: either the parameter dictionary for that filter, or the
    /// null object if that filter has no parameters (or if all of its parameters have
    /// their default values). If none of the filters have parameters, or if all their
    /// parameters have default values, the DecodeParms entry may be omitted
    pub(crate) decode_parms: Option<DecodeParams<'a>>,

    /// The file containing the stream data. If this entry is present, the bytes
    /// between stream and endstream shall be ignored. However, the Length entry
    /// should still specify the number of those bytes (usually, there are no bytes
    /// and Length is 0). The filters that are applied to the file data shall be
    /// specified by FFilter and the filter parameters shall be specified by FDecodeParms
    pub f: Option<FileSpecification<'a>>,

    /// The name of a filter to be applied in processing the data found in the stream's
    /// external file, or an array of zero, one or several such names. The same rules
    /// apply as for Filter
    pub f_filter: Option<Vec<FilterKind>>,

    /// A parameter dictionary, or an array of such dictionaries, used by the filters
    /// specified by FFilter. The same rules apply as for DecodeParms
    pub(crate) f_decode_parms: Option<DecodeParams<'a>>,

    /// A non-negative integer representing the number of bytes in the decoded
    /// (defiltered) stream. It can be used to determine, for example, whether enough
    /// disk space is available to write a stream to a file. This value shall be
    /// considered a hint only; for some stream filters, it may not be possible to
    /// determine this value precisely
    pub dl: Option<usize>,

    pub other: Dictionary<'a>,
}

fn get_filters<'a>(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Vec<FilterKind>> {
    let obj = resolver.resolve(obj)?;

    if let Ok(arr) = resolver.assert_arr(obj.clone()) {
        arr.into_iter()
            .map(|obj| FilterKind::from_str(&resolver.assert_name(obj)?))
            .collect()
    } else {
        Ok(vec![FilterKind::from_str(&resolver.assert_name(obj)?)?])
    }
}

impl<'a> StreamDict<'a> {
    #[track_caller]
    pub fn from_dict(mut dict: Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let len = dict.expect_integer("Length", resolver)? as usize;

        let filter = dict
            .get_object("Filter", resolver)?
            .map(|obj| get_filters(obj, resolver))
            .transpose()?;
        let decode_parms = dict
            .get_object("DecodeParms", resolver)?
            .map(|obj| DecodeParams::from_obj(obj, resolver))
            .transpose()?;
        let f = dict.get::<FileSpecification>("F", resolver)?;
        let f_filter = dict
            .get_object("FFilter", resolver)?
            .map(|obj| get_filters(obj, resolver))
            .transpose()?;
        let f_decode_parms = dict
            .get_object("FDecodeParms", resolver)?
            .map(|obj| DecodeParams::from_obj(obj, resolver))
            .transpose()?;
        let dl = dict
            .get_unsigned_integer("DL", resolver)?
            .map(|i| i as usize);

        Ok(StreamDict {
            len,
            filter,
            decode_parms,
            f,
            f_filter,
            f_decode_parms,
            dl,
            other: dict,
        })
    }
}
