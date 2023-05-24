use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, parse_quote, Data, DeriveInput, Expr, GenericArgument,
    LifetimeParam, LitStr, Path, PathArguments, PathSegment, Token, Type, TypePath,
};

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}

fn field_getter(
    name: &Ident,
    ty: &Type,
    key: &LitStr,
    default: &Option<Expr>,
) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(TypePath { path, .. }) if path.segments.last().unwrap().ident == "Option" => {
            assert!(default.is_none());
            let generic = extract_type_from_option(ty).unwrap();
            quote!(
                let #name = dict.get::<#generic>(#key, resolver)?;
            )
        }
        _ => {
            if let Some(default) = default {
                quote!(
                    let #name = dict.get::<#ty>(#key, resolver)?.unwrap_or(#default);
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

pub fn pdf_obj_inner(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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
                .unwrap();

            let nested = field_attr.parse_args_with(HelperArgs::parse).unwrap();

            let key = nested.key;
            let default = nested.default;

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
        quote!(
            crate::assert_empty(dict);

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

    quote!(
        impl<#(#lifetimes,)* #(#type_params,)* #from_obj_lt> crate::FromObj<'from_obj> for #name #ty_generics #where_clause {
            fn from_obj(obj: crate::Object<'from_obj>, resolver: &mut dyn crate::Resolve<'from_obj>) -> crate::PdfResult<Self> {
                let mut dict = resolver.assert_dict(obj)?;

                #(
                    #getters
                )*

                #return_val
            }
        }
    )
    .into()
}
