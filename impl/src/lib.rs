use quote::{ToTokens};
use syn::{parse_macro_input, parse_quote, Item};

fn process_item(item: &mut Item) {
    if let Item::Fn(item_fn) = item {
        process_itemfn(item_fn);
    }
}

fn process_itemfn(item_fn: &mut syn::ItemFn) {
    let syn::ItemFn{sig, block, ..} = item_fn;
    process_sig_block(sig, block);
}

fn process_sig_block(sig: &mut syn::Signature, block: &mut syn::Block) {
    let syn::Block{stmts, ..} = block;
    process_sig_stmts(sig, stmts);
}

fn process_sig_stmts(sig: &mut syn::Signature, stmts: &mut Vec<syn::Stmt>) {
    let syn::Signature{inputs, output, ..} = sig;
    let _ = inputs;
    let _ = output;
    let use_stmt: syn::Stmt = parse_quote!(use wye_impl::arg;);
    stmts.insert(0, use_stmt);
}

#[proc_macro_attribute]
pub fn wye(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let mut input = parse_macro_input!(input as Item);
    process_item(&mut input);
    let tokens = input.into_token_stream();
    tokens.into()
}

#[proc_macro_attribute]
pub fn arg(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    input
}