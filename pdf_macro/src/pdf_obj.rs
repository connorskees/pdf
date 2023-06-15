use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, parse_quote, Data, DeriveInput, Expr, LifetimeParam, LitStr,
    Token, Type, TypePath,
};

use crate::util::extract_type_from_option;

fn field_getter(name: &Ident, ty: &Type, key: &LitStr, default: &Option<Expr>) -> TokenStream2 {
    if name.to_string() == "stream" {
        return TokenStream2::new();
    }

    match ty {
        Type::Path(TypePath { path, .. }) if path.segments.last().unwrap().ident == "Option" => {
            if default.is_some() {
                panic!("expected field with default to not be optional");
            }

            let generic = extract_type_from_option(ty).unwrap();
            quote!(
                let #name = dict.get::<#generic>(#key, resolver)?;
            )
        }
        _ => {
            if let Some(default) = default {
                quote!(
                    let #name = dict.get::<#ty>(#key, resolver)?.unwrap_or_else(|| #default);
                )
            } else {
                quote!(
                    let #name = dict.expect::<#ty>(#key, resolver)?;
                )
            }
        }
    }
}

struct PdfDictObjField {
    name: Ident,
    ty: Type,
    key: LitStr,
    default: Option<Expr>,
}

struct HelperArgs {
    key: LitStr,
    default: Option<Expr>,
}

impl Parse for HelperArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;

        let default = if input.is_empty() {
            None
        } else {
            let _comma: Token![,] = input.parse()?;
            let _default_ident = input.parse::<Ident>()?;
            let _eq = input.parse::<Token![=]>()?;
            Some(input.parse::<Expr>()?)
        };

        Ok(HelperArgs { key, default })
    }
}

fn obj_type(input: &DeriveInput) -> Option<(TokenStream2, TokenStream2)> {
    let name = &input.ident;
    let generics = &input.generics;

    let obj_type_args: HelperArgs = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("obj_type"))?
        .parse_args()
        .unwrap();

    let obj_type_value: LitStr = obj_type_args.key;
    let obj_subtype_value = obj_type_args.default;

    let mut obj_type = quote!(
        dict.expect_type(#obj_type_value, resolver, false).context(stringify!(#name))?;
    );

    if let Some(subtype) = &obj_subtype_value {
        obj_type.extend(quote!(
            dict.expect_name_is_value("Subtype", #subtype, false, resolver).context(stringify!(#name))?;
        ));
    }

    let type_params = generics.type_params();
    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let subtype_impl = obj_subtype_value.map(|sub| {
        quote!(
            pub const SUBTYPE: &'static str = #sub;
        )
    });
    let obj_type_impl = quote!(
        impl<#(#lifetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
            pub const TYPE: &'static str = #obj_type_value;
            #subtype_impl
        }
    );

    Some((obj_type, obj_type_impl))
}

pub fn pdf_obj_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (obj_type, obj_type_impl) = obj_type(&input)
        .map(|(ty, imp)| (Some(ty), Some(imp)))
        .unwrap_or((None, None));

    let name = input.ident;
    let mut generics = input.generics;

    let fields = match input.data {
        Data::Struct(data_struct) => data_struct.fields.into_iter().map(|field| {
            let name = field.ident.unwrap();
            let ty = field.ty;
            let field_attr = field
                .attrs
                .into_iter()
                .find(|attr| attr.path().is_ident("field"))
                .expect(&format!(
                    "`{}` does not have field decorator",
                    name.to_string()
                ));

            let (key, default) = if name.to_string() == "other" || name.to_string() == "stream" {
                (LitStr::new("", Span::call_site()), None)
            } else {
                let nested = field_attr.parse_args_with(HelperArgs::parse).unwrap();

                (nested.key, nested.default)
            };

            PdfDictObjField {
                name,
                ty,
                key,
                default,
            }
        }),
        _ => todo!(),
    }
    .collect::<Vec<PdfDictObjField>>();

    let mut field_name = fields.iter().map(|v| &v.name).collect::<Vec<_>>();
    let field_type = fields.iter().map(|v| &v.ty).collect::<Vec<_>>();
    let field_key = fields.iter().map(|v| &v.key).collect::<Vec<_>>();
    let field_default = fields.iter().map(|v| &v.default).collect::<Vec<_>>();

    let has_other = field_name.iter().any(|field| field.to_string() == "other");
    let has_stream = field_name.iter().any(|field| field.to_string() == "stream");

    let return_val = if has_other {
        field_name = field_name
            .into_iter()
            .filter(|field| field.to_string() != "other")
            .collect();
        quote!(
            Ok(Self {
                #(
                    #field_name,
                )*
                other: dict,
            })
        )
    } else {
        // todo: clone is superfluous, let's replace with macro
        quote!(
            crate::assert_empty(dict.clone());

            Ok(Self {
                #(
                    #field_name,
                )*
            })
        )
    };

    let getters = field_name
        .iter()
        .zip(field_type.iter())
        .zip(field_key.iter())
        .zip(field_default.iter())
        .map(|(((name, ty), key), default)| field_getter(name, ty, key, default));

    let mut from_obj_lt: LifetimeParam = parse_quote!('from_obj);
    for lt in generics.lifetimes_mut() {
        lt.bounds.insert(0, parse_quote!('from_obj));
        from_obj_lt.bounds.insert(0, lt.lifetime.clone());
    }
    let type_params = generics.type_params();
    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let dict_decl = if has_stream {
        quote!(
            let mut stream = resolver.assert_stream(obj)?;
            let dict = &mut stream.dict.other;
        )
    } else {
        quote!(
            let mut dict = resolver.assert_dict(obj)?;
        )
    };

    quote!(
        impl<#(#lifetimes,)* #(#type_params,)* #from_obj_lt> crate::FromObj<'from_obj> for #name #ty_generics #where_clause {
            fn from_obj(obj: crate::Object<'from_obj>, resolver: &mut dyn crate::Resolve<'from_obj>) -> crate::PdfResult<Self> {
                use anyhow::Context;
                #dict_decl

                #obj_type

                #(
                    #getters
                )*

                #return_val
            }
        }

        #obj_type_impl
    )
    .into()
}
