mod pdf_enum;
mod pdf_obj;

use pdf_enum::pdf_enum_inner;
use pdf_obj::pdf_obj_inner;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn pdf_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    pdf_enum_inner(attr, item)
}

#[proc_macro_derive(FromObj, attributes(field, obj_type))]
pub fn pdf_obj(item: TokenStream) -> TokenStream {
    pdf_obj_inner(item)
}
