use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::channel_protocol::Root;

struct HandlerRenderer<'a> {
    root: &'a Root,
}

impl ToTokens for HandlerRenderer<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Root { vis, ident, items } = self.root;
        let handler_ident = quote::format_ident!("Handle{}", ident);

        tokens.extend(quote! {
            #vis trait #handler_ident {
                #( #items )*
            }
        });
    }
}

pub fn build(root: &Root) -> TokenStream {
    let handle_trait = HandlerRenderer { root };
    quote! {
        // #handle_trait
    }
}
