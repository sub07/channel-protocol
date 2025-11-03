mod channel_protocol;
mod client;
mod enum_message;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn channel_protocol(_attr_content: TokenStream, item: TokenStream) -> TokenStream {
    channel_protocol::build(item.into()).into()
}
