use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, ReturnType, punctuated::Punctuated, token::Comma};

use crate::{
    channel_protocol::{Root, TraitItem, message_struct_name},
    enum_message,
};

struct HandleTraitRenderer<'a> {
    root: &'a Root,
}

impl ToTokens for HandleTraitRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Root { vis, ident, items } = self.root;
        let handler_ident = name(ident);
        let items = items
            .iter()
            .map(|item| HandleTraitItemRenderer { item })
            .collect::<Vec<_>>();

        let dispatch_method = DispatchMethodRenderer { root: self.root };

        tokens.extend(quote! {
            #vis trait #handler_ident <S = ()> {
                #( #items )*
                #dispatch_method
            }
        });
    }
}

struct HandleTraitItemRenderer<'a> {
    item: &'a TraitItem,
}

impl ToTokens for HandleTraitItemRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TraitItem {
            ident,
            args,
            output,
        } = self.item;

        tokens.extend(quote! {
            fn #ident(&mut self, state: S, #args) #output;
        });
    }
}

struct DispatchMethodRenderer<'a> {
    root: &'a Root,
}

impl ToTokens for DispatchMethodRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Root { ident, items, .. } = self.root;
        let enum_message_ident = enum_message::name(ident);

        let dispatch_arms = items.iter().map(|item| DispatchItemRenderer {
            item,
            enum_message_ident: &enum_message_ident,
        });

        tokens.extend(quote! {
            fn dispatch(
                &mut self,
                state: S,
                message: #enum_message_ident,
            ) {
                match message {
                    #( #dispatch_arms )*
                }
            }
        });
    }
}

struct DispatchItemRenderer<'a, 'b> {
    item: &'a TraitItem,
    enum_message_ident: &'b Ident,
}

impl ToTokens for DispatchItemRenderer<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TraitItem {
            ident,
            args,
            output,
        } = self.item;
        let enum_message_ident = self.enum_message_ident;
        let message_variant_ident =
            quote::format_ident!("{}", ident.to_string().to_case(convert_case::Case::Pascal));

        let has_return = matches!(output, ReturnType::Type(_, _));
        let has_arguments = !args.is_empty();

        match (has_arguments, has_return) {
            (false, false) => {
                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident => {
                        self.#ident(state);
                    }
                });
            }
            (false, true) => {
                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident(tx) => {
                        let ret = self.#ident(state);
                        tx.send(ret).unwrap();
                    }
                });
            }
            (true, false) => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = message_struct_name(self.item);

                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }) => {
                        self.#ident(state, #arg_idents);
                    }
                });
            }
            (true, true) => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = message_struct_name(self.item);
                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }, tx) => {
                        let ret = self.#ident(state, #arg_idents);
                        tx.send(ret).unwrap();
                    }
                });
            }
        }

        tokens.extend(quote! {});
    }
}

fn name(ident: &Ident) -> Ident {
    quote::format_ident!("Handle{}", ident)
}

pub fn build(root: &Root) -> TokenStream {
    let handle_trait = HandleTraitRenderer { root };
    quote! {
        #handle_trait
    }
}
