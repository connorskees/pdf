use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    braced, parse::Parse, parse_macro_input, punctuated::Punctuated, token, Lit, Token, Visibility,
};

struct PdfEnumVariant {
    attrs: Vec<syn::Attribute>,
    name: Ident,
    #[allow(dead_code)]
    tok_eq: Token![=],
    value: Lit,
}

impl Parse for PdfEnumVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(PdfEnumVariant {
            attrs: input.call(syn::Attribute::parse_outer)?,
            name: input.parse()?,
            tok_eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct PdfEnum {
    attrs: Vec<syn::Attribute>,
    vis: Visibility,
    #[allow(dead_code)]
    kw_enum: Token![enum],
    name: Ident,
    #[allow(dead_code)]
    tok_brace: token::Brace,
    variants: Punctuated<PdfEnumVariant, Token![,]>,
}

impl Parse for PdfEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(PdfEnum {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse()?,
            kw_enum: input.parse()?,
            name: input.parse()?,
            tok_brace: braced!(content in input),
            variants: content.parse_terminated(PdfEnumVariant::parse, Token![,])?,
        })
    }
}

pub fn pdf_enum_inner(attr: TokenStream, item: TokenStream) -> TokenStream {
    let object_type = parse_macro_input!(attr as Option<Ident>)
        .unwrap_or_else(|| Ident::new("Name", Span::call_site()));
    let item = parse_macro_input!(item as PdfEnum);

    let PdfEnum {
        vis,
        name,
        variants,
        attrs,
        ..
    } = item;

    let field_attrs = variants.iter().map(|v| &v.attrs).collect::<Vec<_>>();
    let field_names = variants.iter().map(|v| &v.name).collect::<Vec<_>>();
    let field_values = variants.iter().map(|v| &v.value).collect::<Vec<_>>();
    
    // temporary method impl during transition to proc macros
    let old_impl = if object_type != Ident::new("Integer", Span::call_site()) {
        quote!(impl #name {
            pub fn from_str(s: &str) -> crate::PdfResult<Self> {
                Ok(match s {
                    #(#field_values => Self::#field_names),*,
                    _ => anyhow::bail!(crate::ParseError::UnrecognizedVariant {
                        ty: stringify!(#name),
                        found: s.to_owned(),
                    })
                })
            }
        })
    } else {
        quote!(impl #name {
            pub fn from_integer(s: i32) -> crate::PdfResult<Self> {
                Ok(match s {
                    #(#field_values => Self::#field_names),*,
                    _ => anyhow::bail!(crate::ParseError::UnrecognizedVariant {
                        ty: stringify!(#name),
                        found: s.to_string(),
                    })
                })
            }
        })
    };

    let field = if object_type != Ident::new("Integer", Span::call_site()) {
        quote!(
            #(
                #(#field_attrs)*
                #field_names,
            )*
        )
    } else {
        quote!(
            #(
                #(#field_attrs)*
                #field_names = #field_values,
            )*
        )
    };

    quote!(
        #(#attrs)*
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        #vis enum #name {
            #field
        }

        impl<'a> crate::FromObj<'a> for #name {
            fn from_obj(obj: crate::Object<'a>, resolver: &mut dyn crate::Resolve<'a>) -> crate::PdfResult<Self> {
                Ok(match obj {
                    #(crate::Object::#object_type(v) if v == #field_values => Self::#field_names,)*
                    crate::Object::#object_type(v) => anyhow::bail!("unrecognized variant {:#?} for {:?}", v, stringify!(#object_type)),
                    _ => anyhow::bail!("invalid object type {:#?} (expected {:?})", obj, stringify!(#object_type)),
                })
            }
        }

        #old_impl        
    )
    .into()
}
