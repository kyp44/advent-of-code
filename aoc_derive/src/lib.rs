extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

struct DebugParams(syn::Ident);
impl syn::parse::Parse for DebugParams {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let type1 = content.parse()?;
        Ok(DebugParams(type1))
    }
}

#[proc_macro_derive(CharGridDebug, attributes(generic))]
pub fn char_grid_debug(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("CharGridDebug item is not a struct or enum");
    let name = ast.ident;

    if let Some(a) = ast
        .attrs
        .iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "generic")
        .next()
    {
        let params: DebugParams =
            syn::parse2(a.tokens.clone()).expect("Invalid generic attribute!");
    }

    let output = quote! {
    impl std::fmt::Debug for #name<bool> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out_fmt(f)
            }
    }
    };

    // TODO test code
    let ast: DeriveInput = syn::parse(output.into()).unwrap();

    let output = quote! {
    impl std::fmt::Debug for #name<bool> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out_fmt(f)
            }
    }
    };

    output.into()
}
