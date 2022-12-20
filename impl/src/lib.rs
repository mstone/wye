use quote::{ToTokens};
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn wye(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as Item);
    let tokens = input.into_token_stream();
    tokens.into()
}