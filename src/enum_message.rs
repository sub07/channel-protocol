use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Field;
use syn::FieldMutability;
use syn::FieldsNamed;
use syn::Ident;
use syn::ReturnType;
use syn::Type;
use syn::Visibility;
use syn::punctuated::Punctuated;
use syn::token::Colon;
use syn::token::Comma;
use syn::{Fields, Variant};

use crate::channel_protocol::FnArg;
use crate::channel_protocol::return_field_ident;
use crate::channel_protocol::{Root, TraitItem};

fn arg_to_fields(arg: &FnArg) -> Field {
    Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: FieldMutability::None,
        ident: Some(arg.ident.clone()),
        colon_token: Some(syn::token::Colon::default()),
        ty: arg.ty.clone(),
    }
}

fn item_to_variant(item: &TraitItem) -> Variant {
    let mut fields = Punctuated::<Field, Comma>::new();
    fields.extend(item.args.iter().map(arg_to_fields));

    if let ReturnType::Type(_, ty) = &item.output {
        fields.push(Field {
            attrs: Vec::new(),
            vis: Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: Some(return_field_ident()),
            colon_token: Some(Colon::default()),
            ty: Type::Verbatim(quote!(oneshot::Sender<#ty>)),
        });
    }

    let fields = if fields.is_empty() {
        Fields::Unit
    } else {
        Fields::Named(FieldsNamed {
            brace_token: syn::token::Brace::default(),
            named: fields,
        })
    };
    Variant {
        attrs: Vec::new(),
        ident: format_ident!("{}", item.ident.to_string().to_case(Case::Pascal)),
        fields,
        discriminant: None,
    }
}

fn variants(items: &[TraitItem]) -> Vec<Variant> {
    items.iter().map(item_to_variant).collect()
}

pub fn name(ident: &Ident) -> Ident {
    format_ident!("{ident}Message")
}

pub fn build(
    Root {
        ident, items, vis, ..
    }: &Root,
) -> TokenStream {
    let enum_name = name(ident);
    let variants = variants(items);

    quote! {
        #[derive(Debug)]
        #vis enum #enum_name {
            #(#variants),*
        }
    }
}
