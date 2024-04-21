/*!
 * https://adobe-type-tools.github.io/font-tech-notes/pdfs/5176.CFF.pdf
 *
 * See also:
 *  - https://adobe-type-tools.github.io/font-tech-notes/pdfs/5177.Type2.pdf
 */

mod charset;
mod charstring;
mod consts;
mod dict;
mod encoding;
mod index;
mod parse;

pub use charset::*;
pub use charstring::CffCharStringInterpreter;
pub use encoding::*;
pub use index::*;
pub use parse::CffParser;

use self::dict::TopDict;

type Offsize = u8;

#[derive(Debug)]
pub struct CffFile<'a> {
    pub(crate) name_index: CffIndex<'a>,
    pub(crate) top_dict: TopDict,
    pub(crate) string_index: CffIndex<'a>,
    pub(crate) charstring_index: CffIndex<'a>,
    pub(crate) encoding: CffEncoding<'a>,
    pub(crate) charset: CffCharset,
}

#[derive(Debug)]
pub struct CffHeader {
    pub major: u8,
    pub minor: u8,
    pub header_size: u8,
    pub off_size: u8,
}
