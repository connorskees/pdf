use std::{borrow::Cow, collections::HashMap, fmt};

use crate::{
    assert_empty,
    error::PdfResult,
    file_specification::FileSpecification,
    flate_decoder::{FlateDecoder, FlateDecoderParams},
    objects::{Dictionary, TypeOrArray},
    Resolve,
};

pub(crate) fn decode_stream(
    stream: &[u8],
    stream_dict: &StreamDict,
    resolver: &mut dyn Resolve,
) -> PdfResult<Vec<u8>> {
    // todo: check if actually Flate
    let decoder_params = FlateDecoderParams::from_dict(
        match &stream_dict.decode_parms {
            Some(TypeOrArray::Type(t)) => t.clone(),
            None => Dictionary::new(HashMap::new()),
            decoder_params => todo!("{:?}", decoder_params),
        },
        resolver,
    )?;

    Ok(FlateDecoder::new(Cow::Borrowed(stream), decoder_params).decode())
}

#[derive(Clone)]
pub struct Stream {
    pub(crate) dict: StreamDict,
    pub(crate) stream: Vec<u8>,
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stream")
            .field("dict", &self.dict)
            .field("stream", &format!("[ {} bytes ]", self.stream.len()))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct StreamDict {
    pub len: usize,
    pub filter: Option<TypeOrArray<String>>,
    pub decode_parms: Option<TypeOrArray<Dictionary>>,
    pub f: Option<FileSpecification>,
    pub f_filter: Option<TypeOrArray<String>>,
    pub f_decode_parms: Option<TypeOrArray<Dictionary>>,
    pub dl: Option<usize>,
}

impl StreamDict {
    #[track_caller]
    pub fn from_dict(mut dict: Dictionary, lexer: &mut impl Resolve) -> PdfResult<StreamDict> {
        let len = dict.expect_integer("Length", lexer)? as usize;

        let filter = dict.get_type_or_arr("Filter", lexer, Resolve::assert_name)?;
        let decode_parms = dict.get_type_or_arr("DecodeParms", lexer, Resolve::assert_dict)?;
        let f = dict
            .get_object("F", lexer)?
            .map(|obj| FileSpecification::from_obj(obj, lexer))
            .transpose()?;
        let f_filter = dict.get_type_or_arr("FFilter", lexer, Resolve::assert_name)?;
        let f_decode_parms = dict.get_type_or_arr("FDecodeParms", lexer, Resolve::assert_dict)?;
        let dl = dict.get_integer("DL", lexer)?.map(|i| i as usize);

        assert_empty(dict);

        Ok(StreamDict {
            len,
            filter,
            decode_parms,
            f,
            f_filter,
            f_decode_parms,
            dl,
        })
    }
}
