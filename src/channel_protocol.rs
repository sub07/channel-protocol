use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, parse::Parse, punctuated::Punctuated};

use crate::{client, enum_message, handler};

#[derive(Debug)]
pub struct Root {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub items: Vec<TraitItem>,
}

impl Parse for Root {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis: syn::Visibility = input.parse()?;
        let _: syn::Token![trait] = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = syn::braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self { vis, ident, items })
    }
}

#[derive(Debug)]
pub struct TraitItem {
    pub ident: syn::Ident,
    pub args: syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
    pub output: syn::ReturnType,
}

impl Parse for TraitItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: syn::Token![fn] = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let args = Punctuated::parse_terminated(&content)?;
        let output: syn::ReturnType = input.parse()?;
        let _: syn::Token![;] = input.parse()?;
        Ok(Self {
            ident,
            args,
            output,
        })
    }
}

impl ToTokens for TraitItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            ident,
            args,
            output,
        } = self;
        tokens.extend(quote! {
            fn #ident(#args) #output;
        });
    }
}

#[derive(Debug)]
pub struct FnArg {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

impl Parse for FnArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;
        let ty: syn::Type = input.parse()?;
        Ok(Self { ident, ty })
    }
}

impl ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { ident, ty, .. } = self;
        tokens.extend(quote! {
            #ident: #ty
        });
    }
}

pub fn message_struct_name(item: &TraitItem) -> Ident {
    format_ident!(
        "{}ParamMessage",
        item.ident.to_string().to_case(Case::Pascal)
    )
}

pub fn build(item: TokenStream) -> TokenStream {
    let root = syn::parse2::<Root>(item)
        .expect("Wrong syntax. An fully valid trait declaration is expected");

    let message_enum = enum_message::build(&root);
    let client = client::build(&root);
    let handler = handler::build(&root);

    quote! {
        #message_enum
        #client
        #handler
    }
}
