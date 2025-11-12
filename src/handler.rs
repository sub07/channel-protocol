use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
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
        let Protocol { vis, messages, .. } = self.protocol;
        let handler_ident_with_state = format_ident!("Handle{}WithState", self.protocol.ident);
        let handler_ident_without_state = format_ident!("Handle{}", self.protocol.ident);

        let messages_with_state = messages
            .iter()
            .map(|message| HandleProtocolMessageRenderer {
                message,
                with_state: true,
            })
            .collect::<Vec<_>>();

        let messages_without_state = messages
            .iter()
            .map(|message| HandleProtocolMessageRenderer {
                message,
                with_state: false,
            })
            .collect::<Vec<_>>();

        let dispatch_method_with_state = DispatchMethodRenderer {
            protocol: self.protocol,
            with_state: true,
        };

        let dispatch_method_without_state = DispatchMethodRenderer {
            protocol: self.protocol,
            with_state: false,
        };

        tokens.extend(quote! {
            #vis trait #handler_ident_with_state <S = ()> {
                #( #messages_with_state )*
                #dispatch_method_with_state
            }

            #vis trait #handler_ident_without_state {
                #( #messages_without_state )*
                #dispatch_method_without_state
            }
        });
    }
}

struct HandleProtocolMessageRenderer<'a> {
    message: &'a ProtocolMessage,
    with_state: bool,
}

impl ToTokens for HandleProtocolMessageRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProtocolMessage {
            ident,
            args,
            output,
        } = self.message;

        let args = if args.is_empty() {
            quote! {}
        } else {
            quote! { , #args }
        };

        tokens.extend(if self.with_state {
            quote! {
                fn #ident(&mut self #args, state: S) #output;
            }
        } else {
            quote! {
                fn #ident(&mut self #args) #output;
            }
        });
    }
}

struct DispatchMethodRenderer<'a> {
    protocol: &'a Protocol,
    with_state: bool,
}

impl ToTokens for DispatchMethodRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Protocol { messages, .. } = self.protocol;
        let enum_message_ident = self.protocol.message_enum_ident();

        let dispatch_arms = messages.iter().map(|message| DispatchMessageRenderer {
            message,
            enum_message_ident: &enum_message_ident,
            with_state: self.with_state,
        });

        tokens.extend(if self.with_state {
            quote! {
                fn _dispatch_with_state(
                    &mut self,
                    message: #enum_message_ident,
                    state: S,
                ) {
                    match message {
                        #( #dispatch_arms )*
                    }
                }

                fn dispatch_with_state(
                    &mut self,
                    message: #enum_message_ident,
                    state: S,
                ) {
                    self._dispatch_with_state(message, state);
                }
            }
        } else {
            quote! {
                fn _dispatch(
                    &mut self,
                    message: #enum_message_ident,
                ) {
                    match message {
                        #( #dispatch_arms )*
                    }
                }

                fn dispatch(
                    &mut self,
                    message: #enum_message_ident,
                ) {
                    self._dispatch(message);
                }
            }
        });
    }
}

struct DispatchMessageRenderer<'a, 'b> {
    message: &'a ProtocolMessage,
    enum_message_ident: &'b Ident,
    with_state: bool,
}

impl ToTokens for DispatchMessageRenderer<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ProtocolMessage { ident, args, .. } = self.message;
        let enum_message_ident = self.enum_message_ident;
        let message_variant_ident =
            quote::format_ident!("{}", ident.to_string().to_case(convert_case::Case::Pascal));

        match self.message.signature_kind() {
            MessageSignatureKind::None => {
                tokens.extend(if self.with_state {
                    quote! {
                        #enum_message_ident::#message_variant_ident => {
                            self.#ident(state);
                        }
                    }
                } else {
                    quote! {
                        #enum_message_ident::#message_variant_ident => {
                            self.#ident();
                        }
                    }
                });
            }
            MessageSignatureKind::OnlyReturn => {
                tokens.extend(if self.with_state {
                    quote! {
                        #enum_message_ident::#message_variant_ident(tx) => {
                            let ret = self.#ident(state);
                            tx.send(ret).unwrap();
                        }
                    }
                } else {
                    quote! {
                        #enum_message_ident::#message_variant_ident(tx) => {
                            let ret = self.#ident();
                            tx.send(ret).unwrap();
                        }
                    }
                });
            }
            MessageSignatureKind::OnlyParam => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = self.message.struct_ident();

                tokens.extend(if self.with_state {
                    quote! {
                        #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }) => {
                            self.#ident(#arg_idents, state);
                        }
                    }
                } else {
                    quote! {
                        #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }) => {
                            self.#ident(#arg_idents);
                        }
                    }
                });
            }
            MessageSignatureKind::ParamReturn => {
                let arg_idents = args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();
                let message_struct_ident = self.message.struct_ident();
                tokens.extend(if self.with_state {
                    quote! {
                        #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }, tx) => {
                            let ret = self.#ident(#arg_idents, state);
                            tx.send(ret).unwrap();
                        }
                    }
                } else {
                    quote! {
                        #enum_message_ident::#message_variant_ident(#message_struct_ident { #arg_idents }, tx) => {
                            let ret = self.#ident(#arg_idents);
                            tx.send(ret).unwrap();
                        }
                    }
                });
            }
        }

        tokens.extend(quote! {});
    }
}

pub fn build(protocol: &Protocol) -> TokenStream {
    let handle_trait = HandleTraitRenderer { protocol };
    quote! {
        #handle_trait
    }
}
