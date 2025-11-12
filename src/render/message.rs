use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{Ident, ReturnType};

use crate::channel_protocol::ProtocolMessage;

pub enum MessageSignatureKind {
    None,
    OnlyReturn,
    OnlyParam,
    ParamReturn,
}

impl ProtocolMessage {
    pub fn struct_ident(&self) -> Ident {
        format_ident!("{}ParamMessage", self.pascal_case_ident())
    }

    pub fn pascal_case_ident(&self) -> Ident {
        format_ident!("{}", self.ident.to_string().to_case(Case::Pascal))
    }

    pub fn signature_kind(&self) -> MessageSignatureKind {
        match (
            !self.args.is_empty(),
            matches!(self.output, ReturnType::Type(_, _)),
        ) {
            (false, false) => MessageSignatureKind::None,
            (false, true) => MessageSignatureKind::OnlyReturn,
            (true, false) => MessageSignatureKind::OnlyParam,
            (true, true) => MessageSignatureKind::ParamReturn,
        }
    }
}
