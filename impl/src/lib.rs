use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

use quote::{ToTokens, format_ident};
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
    let _ = output;

    let mut result = vec![];

    result.push(parse_quote!(let WYE = get_wye(); ));

    for input in inputs.iter() {
        if let syn::FnArg::Typed(syn::PatType{pat, ..}) = input {
            if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                let qvar = format!("{}", pat_ident.ident);
                let hash_var = hash(&pat_ident.ident.to_string());
                let qnode = format_ident!("node_{}", hash_var);
                result.push(parse_quote!(
                    let #qnode = WYE.node(#hash_var, Some(#qvar), #pat);
                ));
            }
        }
    }

    for stmt in stmts.iter() {
        process_stmt(&mut result, inputs, stmt);
    }

    std::mem::swap(stmts, &mut result);
}

fn hash<T: Hash>(t: T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

fn process_stmt(result: &mut Vec<syn::Stmt>, _inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>, stmt: &syn::Stmt) {
    if let syn::Stmt::Expr(expr) = stmt {
        if let syn::Expr::Binary(expr_binary) = expr {
            let syn::ExprBinary{left, right, op, ..} = expr_binary;
            let (left, right): (&syn::Expr, &syn::Expr) = (left, right);
            if let syn::Expr::Path(syn::ExprPath{path: left_path, ..}) = left {
                if let Some(left_ident) = left_path.get_ident() {
                    if let syn::Expr::Path(syn::ExprPath{path: right_path, ..}) = right {
                        if let Some(right_ident) = right_path.get_ident() {
                            let hash_left = hash(left_ident.to_string());
                            let hash_right = hash(right_ident.to_string());
                            let hash_op = hash(op);
                            let hash_expr = hash(expr_binary);

                            let qop = format!("{}", op.to_token_stream());

                            let node_ql = format_ident!("r#node_{}", format!("{}", hash_left));
                            let node_qr = format_ident!("r#node_{}", format!("{}", hash_right));
                            let node_qop = format_ident!("r#node_{}", format!("{}", hash_op));
                            let node_expr = format_ident!("r#node_{}", format!("{}", hash_expr));

                            // let qop = "foo";
                            let stmts: Vec<syn::Stmt> = parse_quote!(
                                let #node_qop = WYE.node(#hash_op, None::<String>, #qop);
                                let r = #expr;
                                let #node_expr = WYE.node(#hash_expr, None::<String>, r);
                                WYE.edge(#node_ql, #node_qop);
                                WYE.edge(#node_qr, #node_qop);
                                WYE.edge(#node_qop, #node_expr);
                            );
                            // result.append(&mut stmts);
                            result.push(parse_quote!({
                                #(#stmts)*;
                                r
                            }));
                            return;
                        }
                    }
                }
            }
        }
    }
    result.push(stmt.clone());
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