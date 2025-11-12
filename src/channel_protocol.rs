use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{parse::Parse, punctuated::Punctuated};

use crate::{client, enum_message, handler};

#[derive(Debug)]
pub struct Protocol {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub messages: Vec<ProtocolMessage>,
}

impl Parse for Protocol {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis: syn::Visibility = input.parse()?;
        let _: syn::Token![trait] = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let content;
        let _ = syn::braced!(content in input);
        let mut messages = Vec::new();
        while !content.is_empty() {
            messages.push(content.parse()?);
        }
        Ok(Self {
            vis,
            ident,
            messages,
        })
    }
}

#[derive(Debug)]
pub struct ProtocolMessage {
    pub ident: syn::Ident,
    pub args: syn::punctuated::Punctuated<ProtocolMessageFnArg, syn::Token![,]>,
    pub output: syn::ReturnType,
}

impl Parse for ProtocolMessage {
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

impl ToTokens for ProtocolMessage {
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
pub struct ProtocolMessageFnArg {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

impl Parse for ProtocolMessageFnArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let _: syn::Token![:] = input.parse()?;
        let ty: syn::Type = input.parse()?;
        Ok(Self { ident, ty })
    }
}

impl ToTokens for ProtocolMessageFnArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { ident, ty, .. } = self;
        tokens.extend(quote! {
            #ident: #ty
        });
    }
}

pub fn build(input: TokenStream) -> TokenStream {
    let protocol = syn::parse2::<Protocol>(input)
        .expect("Wrong syntax. An fully valid trait declaration is expected");

    let message_enum = enum_message::build(&protocol);
    let client = client::build(&protocol);
    let handler = handler::build(&protocol);

    quote! {
        #message_enum
        #client
        #handler
    }
}
