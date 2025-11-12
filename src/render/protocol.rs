use convert_case::{Case, Casing};
use quote::format_ident;

use crate::channel_protocol::Protocol;

impl Protocol {
    pub fn message_enum_ident(&self) -> syn::Ident {
        format_ident!("{}Message", self.ident.to_string().to_case(Case::Pascal))
    }
}
