use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, punctuated::Punctuated, token::Comma};

use crate::{
    channel_protocol::{Protocol, ProtocolMessage},
    render::message::MessageSignatureKind,
};

struct HandleTraitRenderer<'a> {
    protocol: &'a Protocol,
}

impl ToTokens for HandleTraitRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Protocol {
            vis,
            ident,
            messages,
        } = self.protocol;
        let handler_ident = name(ident);
        let messages = messages
            .iter()
            .map(|message| HandleProtocolMessageRenderer { message })
            .collect::<Vec<_>>();

        let dispatch_method = DispatchMethodRenderer {
            protocol: self.protocol,
        };

        tokens.extend(quote! {
            #vis trait #handler_ident <S = ()> {
                #( #messages )*
                #dispatch_method
            }
        });
    }
}

struct HandleProtocolMessageRenderer<'a> {
    message: &'a ProtocolMessage,
}

impl ToTokens for HandleProtocolMessageRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProtocolMessage {
            ident,
            args,
            output,
        } = self.message;

        tokens.extend(quote! {
            fn #ident(&mut self, state: S, #args) #output;
        });
    }
}

struct DispatchMethodRenderer<'a> {
    protocol: &'a Protocol,
}

impl ToTokens for DispatchMethodRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Protocol { messages, .. } = self.protocol;
        let enum_message_ident = self.protocol.message_enum_ident();

        let dispatch_arms = messages.iter().map(|message| DispatchMessageRenderer {
            message,
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

struct DispatchMessageRenderer<'a, 'b> {
    message: &'a ProtocolMessage,
    enum_message_ident: &'b Ident,
}

impl ToTokens for DispatchMessageRenderer<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProtocolMessage { ident, args, .. } = self.message;
        let enum_message_ident = self.enum_message_ident;
        let message_variant_ident =
            quote::format_ident!("{}", ident.to_string().to_case(convert_case::Case::Pascal));

        match self.message.signature_kind() {
            MessageSignatureKind::None => {
                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident => {
                        self.#ident(state);
                    }
                });
            }
            MessageSignatureKind::OnlyReturn => {
                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident(tx) => {
                        let ret = self.#ident(state);
                        tx.send(ret).unwrap();
                    }
                });
            }
            MessageSignatureKind::OnlyParam => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = self.message.struct_ident();

                tokens.extend(quote! {
                    #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }) => {
                        self.#ident(state, #arg_idents);
                    }
                });
            }
            MessageSignatureKind::ParamReturn => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = self.message.struct_ident();
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

pub fn build(protocol: &Protocol) -> TokenStream {
    let handle_trait = HandleTraitRenderer { protocol };
    quote! {
        #handle_trait
    }
}
