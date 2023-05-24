mod pdf_enum;

use pdf_enum::pdf_enum_inner;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn pdf_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    pdf_enum_inner(attr, item)
}
