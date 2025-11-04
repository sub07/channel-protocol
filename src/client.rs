use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    channel_protocol::{Root, TraitItem, message_struct_name},
    enum_message,
};

fn item_to_fn(
    enum_message_name: &Ident,
    item @ TraitItem {
        ident,
        output,
        args,
        ..
    }: &TraitItem,
) -> TokenStream {
    let has_output = matches!(output, syn::ReturnType::Type(_, _));
    let has_arguments = !item.args.is_empty();
    let message_enum_ident = format_ident!("{}", ident.to_string().to_case(Case::Pascal));
    match (has_arguments, has_output) {
        (false, true) => {
            quote! {
                pub fn #ident(&self) #output {
                    let (tx, rx) = oneshot::channel();
                    let message = #enum_message_name::#message_enum_ident(tx);
                    self.0.send(message).unwrap();
                    rx.recv().unwrap()
                }
            }
        }
        (false, false) => {
            quote! {
                pub fn #ident(&self) {
                    let message = #enum_message_name::#message_enum_ident;
                    self.0.send(message).unwrap();
                }
            }
        }
        (true, false) => {
            let fields = args.iter().map(|arg| &arg.ident).collect_vec();
            let message_struct_name = message_struct_name(item);
            quote! {
                pub fn #ident(&self, #args) {
                    let message = #enum_message_name::#message_enum_ident(#message_struct_name {
                        #(#fields,)*
                    });
                    self.0.send(message).unwrap();
                }
            }
        }
        (true, true) => {
            let fields = args.iter().map(|arg| &arg.ident).collect_vec();
            let message_struct_name = message_struct_name(item);
            quote! {
                pub fn #ident(&self, #args) #output {
                    let (tx, rx) = oneshot::channel();
                    let message = #enum_message_name::#message_enum_ident(#message_struct_name {
                        #(#fields,)*
                    }, tx);
                    self.0.send(message).unwrap();
                    rx.recv().unwrap()
                }
            }
        }
    }
}

fn functions(enum_message_name: &Ident, items: &[TraitItem]) -> TokenStream {
    items
        .iter()
        .map(|item| item_to_fn(enum_message_name, item))
        .collect()
}

pub fn build(
    Root {
        vis, ident, items, ..
    }: &Root,
) -> TokenStream {
    let client_struct_name = format_ident!("{}Client", ident);
    let message_enum_name = enum_message::name(ident);
    let functions = functions(&message_enum_name, items);

    quote! {
        #[derive(Clone)]
        #vis struct #client_struct_name(std::sync::mpsc::Sender<#message_enum_name>);

        impl #client_struct_name {
            fn new() -> (Self, std::sync::mpsc::Receiver<#message_enum_name>) {
                let (sender, receiver) = std::sync::mpsc::channel();
                (Self(sender), receiver)
            }

            #functions
        }
    }
}
