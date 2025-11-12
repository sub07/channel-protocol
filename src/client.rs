use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    channel_protocol::{Protocol, ProtocolMessage},
    render::message::MessageSignatureKind,
};

fn message_to_fn(
    enum_message_name: &Ident,
    message @ ProtocolMessage {
        ident,
        output,
        args,
        ..
    }: &ProtocolMessage,
) -> TokenStream {
    let message_enum_ident = format_ident!("{}", ident.to_string().to_case(Case::Pascal));
    match message.signature_kind() {
        MessageSignatureKind::None => {
            quote! {
                pub fn #ident(&self) {
                    let message = #enum_message_name::#message_enum_ident;
                    self.0.send(message).unwrap();
                }
            }
        }
        MessageSignatureKind::OnlyReturn => {
            quote! {
                pub fn #ident(&self) #output {
                    let (tx, rx) = oneshot::channel();
                    let message = #enum_message_name::#message_enum_ident(tx);
                    self.0.send(message).unwrap();
                    rx.recv().unwrap()
                }
            }
        }
        MessageSignatureKind::OnlyParam => {
            let fields = args.iter().map(|arg| &arg.ident).collect_vec();
            let message_struct_name = message.struct_ident();
            quote! {
                pub fn #ident(&self, #args) {
                    let message = #enum_message_name::#message_enum_ident(#message_struct_name {
                        #(#fields,)*
                    });
                    self.0.send(message).unwrap();
                }
            }
        }
        MessageSignatureKind::ParamReturn => {
            let fields = args.iter().map(|arg| &arg.ident).collect_vec();
            let message_struct_name = message.struct_ident();
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

fn functions(enum_message_name: &Ident, messages: &[ProtocolMessage]) -> TokenStream {
    messages
        .iter()
        .map(|m| message_to_fn(enum_message_name, m))
        .collect()
}

pub fn build(
    protocol @ Protocol {
        vis,
        ident,
        messages,
        ..
    }: &Protocol,
) -> TokenStream {
    let client_struct_name = format_ident!("{}Client", ident);
    let message_enum_ident = protocol.message_enum_ident();
    let functions = functions(&message_enum_ident, messages);

    quote! {
        #[derive(Clone)]
        #vis struct #client_struct_name(std::sync::mpsc::Sender<#message_enum_ident>);

        impl #client_struct_name {
            fn new() -> (Self, std::sync::mpsc::Receiver<#message_enum_ident>) {
                let (sender, receiver) = std::sync::mpsc::channel();
                (Self(sender), receiver)
            }

            #functions
        }
    }
}
