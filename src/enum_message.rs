use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Field;
use syn::FieldMutability;
use syn::Ident;
use syn::ReturnType;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use crate::channel_protocol::FnArg;
use crate::channel_protocol::message_struct_name;
use crate::channel_protocol::{Root, TraitItem};

struct MessageStructRenderer<'a> {
    item: &'a TraitItem,
}

impl ToTokens for MessageStructRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.item.args.is_empty() {
            return;
        }

        let struct_name = message_struct_name(self.item);
        let fields = self
            .item
            .args
            .iter()
            .map(arg_to_field)
            .collect::<Punctuated<_, Comma>>();

        tokens.extend(quote! {
            struct #struct_name {
                #fields
            }
        });
    }
}

struct MessageEnumRenderer<'a> {
    root: &'a Root,
}

impl ToTokens for MessageEnumRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.root.items.is_empty() {
            return;
        }

        let variants = self
            .root
            .items
            .iter()
            .map(|item| {
                let variant_name =
                    format_ident!("{}", item.ident.to_string().to_case(Case::Pascal));

                let ret_val = match &item.output {
                    ReturnType::Default => None,
                    ReturnType::Type(_, ty) => Some(quote! {
                        oneshot::Sender<#ty>
                    }),
                };

                if item.args.is_empty() {
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
                    let struct_name = message_struct_name(item);
                    quote! {
                        #variant_name(#struct_name, #ret_val)
                    }
                }
            })
            .collect::<Punctuated<_, Comma>>();

        let vis = &self.root.vis;
        let message_enum_name = name(&self.root.ident);

        tokens.extend(quote! {
            #vis enum #message_enum_name {
                #variants
            }
        });
    }
}

fn arg_to_field(arg: &FnArg) -> Field {
    Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: FieldMutability::None,
        ident: Some(arg.ident.clone()),
        colon_token: Some(syn::token::Colon::default()),
        ty: arg.ty.clone(),
    }
}

pub fn name(ident: &Ident) -> Ident {
    format_ident!("{ident}Message")
}

pub fn build(root: &Root) -> TokenStream {
    let message_enum = MessageEnumRenderer { root };
    let message_structs = root.items.iter().map(|item| MessageStructRenderer { item });

    quote! {
        #(#message_structs)*
        #message_enum
    }
}
