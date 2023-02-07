extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::GenericArgument;
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, ItemImpl};

/// TODO
struct GenericParams(Punctuated<GenericArgument, syn::token::Comma>);
impl syn::parse::Parse for GenericParams {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        Ok(GenericParams(
            content.parse_terminated(GenericArgument::parse)?,
        ))
    }
}

/// TODO: After refactoring CharGrid do we even need this?
#[proc_macro_derive(CharGridDebug, attributes(generics))]
pub fn char_grid_debug(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident;

    // Create output impl
    let mut output: ItemImpl = parse_quote! {
    impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.out_fmt(f)
            }
    }
    };

    // Pull out generic attribute if present
    // This implements Debug only for a specific generic
    if let Some(a) = ast
        .attrs
        .iter()
        .find(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "generics")
    {
        let params: GenericParams =
            syn::parse2(a.tokens.clone()).expect("Invalid generic attribute!");

        // Add generics
        if let syn::Type::Path(ref mut p) = *output.self_ty {
            let punctuated = params.0;
            p.path.segments[0].arguments =
                syn::PathArguments::AngleBracketed(parse_quote!(<#punctuated>))
        }
    }

    output.to_token_stream().into()
}
