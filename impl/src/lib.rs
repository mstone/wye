use proc_macro2::{Span};
use quote::{ToTokens};
use syn::{parse_macro_input, Item, Token};

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
    let use_stmt = syn::Stmt::Item(syn::Item::Use(syn::ItemUse{
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        use_token: Token![use](Span::call_site()),
        leading_colon: None,
        tree: syn::UseTree::Path(syn::UsePath{
            ident: syn::Ident::new("wye", Span::call_site()),
            colon2_token: syn::Token![::](Span::call_site()),
            tree: Box::new(syn::UseTree::Name(syn::UseName{ ident: syn::Ident::new("arg", Span::call_site()) }))
        }),
        semi_token: Token![;](Span::call_site())
    }));
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