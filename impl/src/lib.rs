use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

use quote::{ToTokens, format_ident};
use syn::{parse_macro_input, parse_quote, Item, Expr, punctuated::Punctuated, token::Comma};

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
    result.push(parse_quote!(let (FRAME, FRAME_ARGS) = WYE.frame();));

    for (input_slot, input) in inputs.iter().enumerate() {
        if let syn::FnArg::Typed(syn::PatType{pat, ..}) = input {
            if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                let qvar = format!("{}", pat_ident.ident);
                let slot = hash(&pat_ident.ident.to_string());
                result.push(parse_quote!({
                    WYE.node(FRAME, #slot, Some(#qvar), #pat);
                    if let Some((arg_frame, arg_slot)) = FRAME_ARGS.get(#input_slot).copied().flatten() {
                        WYE.edge(arg_frame, arg_slot, FRAME, #slot);
                    }
                }));
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
                            let hash_l = hash(left_ident.to_string());
                            let hash_r = hash(right_ident.to_string());
                            let hash_op = hash(op);
                            let hash_expr = hash(expr_binary);

                            let qop = format!("{}", op.to_token_stream());

                            // let qop = "foo";
                            let stmts: Vec<syn::Stmt> = parse_quote!(
                                WYE.node(FRAME, #hash_op, None::<String>, #qop);
                                let r = #expr;
                                WYE.node(FRAME, #hash_expr, None::<String>, r);
                                WYE.edge(FRAME, #hash_l, FRAME, #hash_op);
                                WYE.edge(FRAME, #hash_r, FRAME, #hash_op);
                                WYE.edge(FRAME, #hash_op, FRAME, #hash_expr);
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

fn process_expr(expr: &mut Expr, stack: &mut Vec<Expr>) {
    if let Expr::Call(call) = expr {
        let args = &mut call.args;
        stack.push(*call.func.clone());

        let has_nested_call = args.iter().any(|arg| {
            matches!(arg, Expr::Call(_))
        });

        if has_nested_call {
            rewrite_expr(expr);
        }

        stack.pop();
    }
}

fn rewrite_expr(expr: &mut Expr) {
    if let Expr::Call(call) = expr {
        let mut stmts: Vec<syn::Stmt> = vec![];
        let mut args2: Punctuated<Expr, Comma> = Punctuated::new();
        stmts.push(parse_quote!(
            WYE.push_frame();
        ));
        for (n, arg) in call.args.iter_mut().enumerate() {
            let argn = format_ident!("arg{}", n);
            if matches!(arg, Expr::Call(_)) {
                rewrite_expr(arg);
                stmts.push(parse_quote!(
                    let #argn = #arg;
                ));
                stmts.push(parse_quote!(
                    WYE.push_var(WYE.last_node());
                ));
            } else {
                stmts.push(parse_quote!(
                    let #argn = #arg;
                ));
                stmts.push(parse_quote!(
                    WYE.push_lit();
                ));
            }
            args2.push(parse_quote!(#argn));
        }
        call.args = args2;
        let mut call2 = parse_quote!({
            let WYE = get_wye();
            #(#stmts)*;
            let ret = #call;
            WYE.pop_frame();
            ret
        });
        std::mem::swap(expr, &mut call2);
    }
}

#[proc_macro_attribute]
pub fn wye(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = args;
    let mut input = parse_macro_input!(input as Item);
    process_item(&mut input);
    let tokens = input.into_token_stream();
    tokens.into()
}

#[proc_macro]
pub fn wyre(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Expr);
    let mut stack = vec![];
    process_expr(&mut input, &mut stack);
    let tokens = input.into_token_stream();
    tokens.into()
}