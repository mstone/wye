use proc_macro::{TokenStream};

#[proc_macro_attribute]
pub fn wye(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    input
}