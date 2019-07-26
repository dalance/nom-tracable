#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::ToTokens;
use syn::{self, parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, Stmt};

/// Custom attribute for tracable parser
#[proc_macro_attribute]
pub fn tracable_parser(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let item = parse_macro_input!(item as ItemFn);
    impl_tracable_parser(&attr, &item)
}

fn impl_tracable_parser(_attr: &AttributeArgs, item: &ItemFn) -> TokenStream {
    let default = impl_tracable_parser_default(&item);
    let trace = impl_tracable_parser_trace(&item);

    let mut item = item.clone();

    item.block.stmts.clear();
    item.block.stmts.push(default);
    item.block.stmts.push(trace);

    item.into_token_stream().into()
}

fn impl_tracable_parser_default(item: &ItemFn) -> Stmt {
    let body = item.block.as_ref();
    parse_quote! {
        #[cfg(not(any(feature = "forward_trace", feature = "backward_trace")))]
        {
            #body
        }
    }
}

fn impl_tracable_parser_trace(item: &ItemFn) -> Stmt {
    let ident = &item.ident;

    let input = if let Some(x) = &item.decl.inputs.first() {
        match x.value() {
            FnArg::Captured(arg) => &arg.pat,
            _ => panic!("function with #[tracable_parser] must have an argument"),
        }
    } else {
        panic!("function with #[tracable_parser] must have an argument");
    };

    let body = item.block.as_ref();

    parse_quote! {
        #[cfg(any(feature = "forward_trace", feature = "backward_trace"))]
        {
            let depth = #input.get_depth();
            #[cfg(feature = "forward_trace")]
            println!(
                "{:<128} : {}",
                format!("{}{}-> {}{}", "\u{001b}[1;37m", " ".repeat(depth), stringify!(#ident), "\u{001b}[0m"),
                #input.format(),
            );
            let #input = #input.inc_depth();
            let body_ret = {
                let body = || { #body };
                body()
            };
            match body_ret {
                Ok((s, x)) => {
                    #[cfg(feature = "backward_trace")]
                    println!(
                        "{:<128} : {}",
                        format!("{}{}<- {}{}", "\u{001b}[1;32m", " ".repeat(depth), stringify!(#ident), "\u{001b}[0m"),
                        s.format(),
                    );
                    Ok((s.dec_depth(), x))
                },
                Err(x) => {
                    #[cfg(feature = "backward_trace")]
                    println!(
                        "{:<128}",
                        format!("{}{}<- {}{}", "\u{001b}[1;31m", " ".repeat(depth), stringify!(#ident), "\u{001b}[0m"),
                    );
                    Err(x)
                },
            }
        }
    }
}
