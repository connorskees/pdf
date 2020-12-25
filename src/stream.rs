use std::fmt;

use crate::{
    error::PdfResult,
    file_specification::FileSpecification,
    objects::{Dictionary, TypeOrArray},
    Lexer,
};

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
    pub decode_params: Option<TypeOrArray<Dictionary>>,
    pub f: Option<FileSpecification>,
    pub f_filter: Option<TypeOrArray<String>>,
    pub f_decode_params: Option<TypeOrArray<Dictionary>>,
    pub dl: Option<usize>,
}

impl StreamDict {
    pub fn from_dict(mut dict: Dictionary, lexer: &mut Lexer) -> PdfResult<StreamDict> {
        let len = dict.expect_integer("Length", lexer)? as usize;

        let filter = dict.get_type_or_arr("Filter", lexer, Lexer::assert_name)?;
        let decode_params = dict.get_type_or_arr("DecodeParms", lexer, Lexer::assert_dict)?;
        let f = dict
            .get_object("F")
            .map(|obj| FileSpecification::from_obj(obj, lexer))
            .transpose()?;
        let f_filter = dict.get_type_or_arr("FFilter", lexer, Lexer::assert_name)?;
        let f_decode_params = dict.get_type_or_arr("FDecodeParms", lexer, Lexer::assert_dict)?;
        let dl = dict.get_integer("DL", lexer)?.map(|i| i as usize);

        if !dict.is_empty() {
            todo!()
        }

        Ok(StreamDict {
            len,
            filter,
            decode_params,
            f,
            f_filter,
            f_decode_params,
            dl,
        })
    }
}
