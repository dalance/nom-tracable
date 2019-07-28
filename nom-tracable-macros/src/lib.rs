#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::ToTokens;
use syn::{self, parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, Stmt};

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
        #[cfg(not(feature = "trace"))]
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
        #[cfg(feature = "trace")]
        {
            let (depth, #input) = nom_tracable::forward_trace(#input, stringify!(#ident));

            let body_ret = {
                let body = || { #body };
                body()
            };

            nom_tracable::backward_trace(body_ret, stringify!(#ident), depth)
        }
    }
}
