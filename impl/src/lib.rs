use std::{hash::{Hash, Hasher}, collections::{hash_map::DefaultHasher, HashMap}};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, TokenStreamExt};
use syn::{parse_macro_input, parse_quote, Item, Expr, punctuated::Punctuated, token::{Comma}, Block, Stmt, Macro, parse2, spanned::Spanned, Ident,};

fn process_item(args: &WyeArgMap, item: &mut Item) {
    if let Item::Fn(item_fn) = item {
        process_itemfn(args, item_fn);
    }
}

fn process_itemfn(args: &WyeArgMap, item_fn: &mut syn::ItemFn) {
    let syn::ItemFn{sig, block, ..} = item_fn;
    process_sig_block(args, sig, block);
}

fn process_sig_block(args: &WyeArgMap, sig: &mut syn::Signature, block: &mut syn::Block) {
    let syn::Block{stmts, ..} = block;
    process_sig_stmts(args, sig, stmts);
}

fn process_sig_stmts(args: &WyeArgMap, sig: &mut syn::Signature, stmts: &mut Vec<syn::Stmt>) {
    let syn::Signature{inputs, output, ..} = sig;
    let _ = output;

    let mut result = vec![];

    result.append(&mut parse_quote!(
        let WYE = get_wye();
        let (FRAME, FRAME_ARGS) = WYE.frame();
    ));

    for (input_slot, input) in inputs.iter().enumerate() {
        if let syn::FnArg::Typed(syn::PatType{pat, ty, ..}) = input {
            if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                let to_string_expr = args.get(&pat_ident.ident).cloned().unwrap_or_else(|| {
                    parse_quote!(|x: &#ty| x.to_string())
                });
                let qvar = format!("{}", pat_ident.ident);
                let slot = hash(pat_ident.ident.to_string());
                result.push(parse_quote!({
                    WYE.node(FRAME, #slot, Some((#qvar).to_string()), ((#to_string_expr)(&#pat)));
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
        match expr {
            syn::Expr::Binary(_) => {
                rewrite_expr_binary(result, expr);
                return;
            }
            syn::Expr::Macro(_) => {
                rewrite_expr_macro(result, expr);
                return;
            }
            _ => (),
        }
    }
    result.push(stmt.clone());
}

#[derive(Debug)]
struct FormatArgs(Punctuated<Expr, Comma>);

impl syn::parse::Parse for FormatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut format_args: Punctuated<Expr, Comma> = Punctuated::new();
        loop {
            format_args.push_value(input.parse()?);
            if input.is_empty() {
                break;
            }
            format_args.push_punct(input.parse()?);
        }
        Ok(FormatArgs(format_args))
    }
}

impl quote::ToTokens for FormatArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(&self.0, Comma::default());
    }
}

fn rewrite_expr_macro(result: &mut Vec<Stmt>, expr: &Expr) {
    if let syn::Expr::Macro(expr_macro) = expr {
        let syn::ExprMacro{mac, ..} = expr_macro;
        let syn::Macro{path, tokens, ..} = mac;
        if let Some(ident) = path.get_ident() {
            let hash_op = hash(ident);
            let hash_expr = hash(expr_macro);

            let qop = format!("{}", expr.to_token_stream());

            let mut stmts: Vec<syn::Stmt> = parse_quote!(
                WYE.node(FRAME, #hash_op, None::<String>, (&#qop).to_string());
                let r = #expr;
                WYE.node(FRAME, #hash_expr, None::<String>, (&r).to_string());
            );
            
            let mut input_stmts: Vec<syn::Stmt> = Vec::new();
            let format_args: FormatArgs = parse2(tokens.clone()).unwrap();
            eprintln!("FORMAT ARGS: {format_args:?}");
            for arg in format_args.0.iter().skip(1) {
                match arg {
                    syn::Expr::Path(syn::ExprPath{path, ..}) => {
                        if let Some(ident) = path.get_ident() {
                            let hash_arg = hash(ident);
                            input_stmts.push(parse_quote!(
                                WYE.edge(FRAME, #hash_arg, FRAME, #hash_op);
                            ));
                        }
                    },
                    syn::Expr::Reference(syn::ExprReference{expr, ..}) => {
                        if let Expr::Path(syn::ExprPath{path, ..}, ..) = &**expr {
                            if let Some(ident) = path.get_ident() {
                                let hash_arg = hash(ident);
                                input_stmts.push(parse_quote!(
                                    WYE.edge(FRAME, #hash_arg, FRAME, #hash_op);
                                ));
                            }
                        }
                    },
                    _ => {}
                }
            }
            stmts.append(&mut input_stmts);
            stmts.push(parse_quote!(
                WYE.edge(FRAME, #hash_op, FRAME, #hash_expr);
            ));
            result.push(parse_quote!({
                #(#stmts)*;
                r
            }));
        }
    }
}

fn rewrite_expr_binary(result: &mut Vec<Stmt>, expr: &syn::Expr) {
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

                        let stmts: Vec<syn::Stmt> = parse_quote!(
                            WYE.node(FRAME, #hash_op, None::<String>, (&#qop).to_string());
                            let r = #expr;
                            WYE.node(FRAME, #hash_expr, None::<String>, (&r).to_string());
                            WYE.edge(FRAME, #hash_l, FRAME, #hash_op);
                            WYE.edge(FRAME, #hash_r, FRAME, #hash_op);
                            WYE.edge(FRAME, #hash_op, FRAME, #hash_expr);
                        );
                        result.push(parse_quote!({
                            #(#stmts)*;
                            r
                        }));
                    }
                }
            }
        }
    }
}

fn process_expr(mut expr: Expr) -> Vec<Stmt> {
    if matches!(expr, Expr::Call(_)) {
        rewrite_expr(&mut expr);
    }
    vec![Stmt::Expr(expr)]
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
            match arg {
                Expr::Call(_) | Expr::Macro(_) => {
                    rewrite_expr(arg);
                    stmts.append(&mut parse_quote!(
                        let #argn = #arg;
                        WYE.push_var(WYE.last_node());
                    ));
                },
                Expr::Reference(_) => {
                    rewrite_expr(arg);
                    let qarg = arg.span().unwrap().source_text();
                    // let qarg: String = arg.to_token_stream().to_string();
                    let slot = hash(&arg);
                    stmts.append(&mut parse_quote!(
                        let #argn = #arg;
                        WYE.node(FRAME, #slot, Some((#qarg).to_string()), (&#argn).to_string());
                        WYE.push_var(WYE.last_node());
                    ));
                },
                Expr::Path(path) if path.path.get_ident().is_some() => {
                    let ident = path.path.get_ident().unwrap();
                    let qident = ident.span().unwrap().source_text();
                    let slot = hash(ident.to_string());
                    stmts.append(&mut parse_quote!(
                        let #argn = #arg;
                        WYE.node(FRAME, #slot, Some((#qident).to_string()), (&#argn).to_string());
                        WYE.push_var(WYE.last_node());
                    ));
                },
                _ => {
                    stmts.append(&mut parse_quote!(
                        let #argn = #arg;
                        WYE.push_lit();
                    ));
                }
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

fn process_inner_item(mut item: Item) -> Vec<Stmt> {
    if let Item::Macro(syn::ItemMacro{ref mut mac, ..}) = &mut item {
        let syn::Macro{path, tokens, ..} = mac;
        eprintln!("MACRO TOKENS: {tokens:?}");
        if path.is_ident("format") {
            rewrite_format_macro(mac);
        }
    }
    vec![Stmt::Item(item)]
}

fn rewrite_format_macro(mac: &mut Macro) {
    // let syn::Macro{path, bang_token, delimiter, tokens} = mac;
    let _ = mac;
}

fn process_local(mut local: syn::Local) -> Vec<Stmt> {
    if let syn::Local{pat: ref pat@syn::Pat::Ident(ref pat_ident), init: Some((_, ref mut expr)), ..} = local {
        rewrite_expr(expr);
        let qvar = format!("{}", pat_ident.ident);
        let slot = hash(pat_ident.ident.to_string());
        let mut epilogue: Vec<Stmt> = parse_quote!({
            let RET = WYE.last_node();
            WYE.node(FRAME, #slot, Some((#qvar).to_string()), (&#pat).to_string());
            WYE.edge(RET.0, RET.1, FRAME, #slot);
        });
        let mut result = vec![Stmt::Local(local)];
        result.append(&mut epilogue);
        result
    } else {
        vec![Stmt::Local(local)]
    }
}

struct Stmts(Vec<Stmt>);

impl syn::parse::Parse for Stmts {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Block::parse_within(input).map(Stmts)
    }
}

impl quote::ToTokens for Stmts {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let brace_token = syn::token::Brace::default();
        brace_token.surround(tokens, |tokens| {
            tokens.append_all(&self.0);
        });
    }
}

struct WyeArg{
    ident: syn::Ident,
    colon_token: syn::Token![:],
    expr: syn::Expr,
}

impl syn::parse::Parse for WyeArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self{
            ident: input.parse()?,
            colon_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl quote::ToTokens for WyeArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

struct WyeArgs(Punctuated<WyeArg, Comma>);

impl syn::parse::Parse for WyeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = Punctuated::new();
        loop {
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
            if input.is_empty() {
                break;
            }
            let comma: Comma = input.parse()?;
            args.push_punct(comma);
        }
        Ok(Self(args))
    }
}

impl quote::ToTokens for WyeArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(self.0.iter(), Comma::default())
    }
}

type WyeArgMap = HashMap<Ident, Expr>;

impl WyeArgs {
    fn process(&self) -> WyeArgMap {
        let mut args = HashMap::new();
        for arg in self.0.iter() {
            args.insert(arg.ident.clone(), arg.expr.clone());
        }
        args
    }
}

#[proc_macro_attribute]
pub fn wye(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = if !args.is_empty() {
        parse_macro_input!(args as WyeArgs).process()
    } else {
        WyeArgMap::new()
    };    let mut input = parse_macro_input!(input as Item);
    process_item(&args, &mut input);
    let tokens = input.into_token_stream();
    tokens.into()
}

#[proc_macro]
pub fn wyre(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Stmts);
    input.0 = input.0.into_iter().flat_map(|stmt| {
        // eprintln!("WYRE {stmt:?}");
        match stmt {
            Stmt::Local(local) => {
                process_local(local)
            },
            Stmt::Expr(expr) => {
                process_expr(expr)
            },
            Stmt::Item(item) => {
                process_inner_item(item)
            },
            _ => {vec![stmt]},
        }
    }).collect::<Vec<_>>();
    input.0.insert(0, parse_quote!(let (FRAME, _) = WYE.frame();));
    input.0.insert(0, parse_quote!(let WYE = get_wye();));
    let tokens = input.into_token_stream();
    tokens.into()
}