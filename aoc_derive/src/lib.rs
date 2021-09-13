extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, GenericArgument, Ident,
    ItemImpl, Token,
};

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
    if let Some(a) = ast
        .attrs
        .iter()
        .filter(|a| a.path.segments.len() == 1 && a.path.segments[0].ident == "generics")
        .next()
    {
        let params: GenericParams =
            syn::parse2(a.tokens.clone()).expect("Invalid generic attribute!");

        // Add generics
        if let syn::Type::Path(ref mut p) = *output.self_ty {
            let x = syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: (),
                args: params.0,
                gt_token: (),
            };
            //p.path.segments[0].arguments = syn::PathArguments
        }
    }

    output.to_token_stream().into()
}
