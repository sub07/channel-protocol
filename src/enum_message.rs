use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Field;
use syn::FieldMutability;
use syn::Ident;
use syn::ReturnType;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use crate::channel_protocol::ProtocolMessageFnArg;
use crate::channel_protocol::{Protocol, ProtocolMessage};
use crate::render::message::MessageSignatureKind;

struct MessageStructDefinitionRenderer<'a> {
    message: &'a ProtocolMessage,
}

fn arg_to_field(arg: &ProtocolMessageFnArg) -> Field {
    Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: FieldMutability::None,
        ident: Some(arg.ident.clone()),
        colon_token: Some(syn::token::Colon::default()),
        ty: arg.ty.clone(),
    }
}

impl ToTokens for MessageStructDefinitionRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.message.args.is_empty() {
            return;
        }

        let struct_name = self.message.struct_ident();
        let fields = self
            .message
            .args
            .iter()
            .map(arg_to_field)
            .collect::<Punctuated<_, Comma>>();

        tokens.extend(quote! {
            #[derive(Debug)]
            struct #struct_name {
                #fields
            }
        });
    }
}

struct MessageDebugImplRenderer<'a> {
    protocol: &'a Protocol,
}

struct MessageMatchArmDebugImplRenderer<'a> {
    message: &'a ProtocolMessage,
    enum_ident: &'a Ident,
}

impl ToTokens for MessageMatchArmDebugImplRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message = self.message;
        let ident = &message.ident;
        let variant_ident = message.pascal_case_ident();
        let enum_ident = self.enum_ident;

        let match_arm = match self.message.signature_kind() {
            MessageSignatureKind::None => {
                quote! {
                    #enum_ident::#variant_ident => write!(f, "{}()", stringify!(#ident)),
                }
            }
            MessageSignatureKind::OnlyReturn => {
                let ret = &message.output;
                quote! {
                    #enum_ident::#variant_ident(_) => write!(f, "{}() {}", stringify!(#ident), stringify!(#ret)),
                }
            }
            MessageSignatureKind::OnlyParam => {
                let struct_param = message.struct_ident();

                let arg_names = message
                    .args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();

                #[allow(clippy::literal_string_with_formatting_args)]
                let punctuated_arg_format = message
                    .args
                    .iter()
                    .map(|_| "{:?}")
                    .collect::<Vec<_>>()
                    .join(", ");

                let format_str = format!("{{}}({punctuated_arg_format})");

                quote! {
                    #enum_ident::#variant_ident(#struct_param { #arg_names }) => {
                        write!(f, #format_str, stringify!(#ident), #arg_names)
                    },
                }
            }
            MessageSignatureKind::ParamReturn => {
                let struct_param = message.struct_ident();
                let ret = &message.output;

                let arg_names = message
                    .args
                    .iter()
                    .map(|arg| &arg.ident)
                    .collect::<Punctuated<_, Comma>>();

                #[allow(clippy::literal_string_with_formatting_args)]
                let punctuated_arg_format = message
                    .args
                    .iter()
                    .map(|_| "{:?}")
                    .collect::<Vec<_>>()
                    .join(", ");

                let format_str = format!("{{}}({punctuated_arg_format}) {{}}");

                quote! {
                    #enum_ident::#variant_ident(#struct_param { #arg_names }, _) => {
                        write!(f, #format_str, stringify!(#ident), #arg_names, stringify!(#ret))
                    },
                }
            }
        };

        tokens.extend(match_arm);
    }
}

impl ToTokens for MessageDebugImplRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message_enum_ident = self.protocol.message_enum_ident();
        let match_arms =
            self.protocol
                .messages
                .iter()
                .map(|message| MessageMatchArmDebugImplRenderer {
                    message,
                    enum_ident: &message_enum_ident,
                });

        tokens.extend(quote! {
            impl std::fmt::Debug for #message_enum_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        });
    }
}

struct MessageEnumDefinitionRenderer<'a> {
    protocol: &'a Protocol,
}

impl ToTokens for MessageEnumDefinitionRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.protocol.messages.is_empty() {
            return;
        }

        let variants = self
            .protocol
            .messages
            .iter()
            .map(|m| {
                let variant_name = m.pascal_case_ident();

                let ret_val = match &m.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, ty) => Some(quote! {
                        oneshot::Sender<#ty>
                    }),
                };

                if m.args.is_empty() {
                    ret_val.map_or_else(
                        || {
                            quote! {
                                #variant_name
                            }
                        },
                        |ret_val| {
                            quote! {
                                #variant_name(#ret_val)
                            }
                        },
                    )
                } else {
                    let struct_name = m.struct_ident();
                    quote! {
                        #variant_name(#struct_name, #ret_val)
                    }
                }
            })
            .collect::<Punctuated<_, Comma>>();

        let vis = &self.protocol.vis;
        let message_enum_name = self.protocol.message_enum_ident();

        tokens.extend(quote! {
            #vis enum #message_enum_name {
                #variants
            }
        });
    }
}

pub fn build(protocol: &Protocol) -> TokenStream {
    let message_enum = MessageEnumDefinitionRenderer { protocol };
    let message_structs = protocol
        .messages
        .iter()
        .map(|m| MessageStructDefinitionRenderer { message: m });
    let enum_debug_impl = MessageDebugImplRenderer { protocol };

    quote! {
        #(#message_structs)*
        #message_enum
        #enum_debug_impl
    }
}
