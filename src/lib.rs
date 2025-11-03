//! # Channel Protocol
//! A procedural macro to generate channel protocol clients.
//! You can use function oriented communication between threads instead of communicating by sending messages through channels.
//! This is an abstraction over channels that makes inter-thread communication easier to use and read.
//!
//! ## Example
//! ```
#![doc = include_str!("../examples/sync.rs")]
//! ```
mod channel_protocol;
mod client;
mod enum_message;

use proc_macro::TokenStream;
/// Expect a trait definition as input and generate a channel protocol based on it.
#[proc_macro_attribute]
pub fn channel_protocol(_attr_content: TokenStream, item: TokenStream) -> TokenStream {
    channel_protocol::build(item.into()).into()
}
