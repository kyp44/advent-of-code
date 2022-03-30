extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Fields;
use syn::GenericArgument;
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, ItemImpl, ItemStruct, Type,
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

// TODO: these will probably go away
#[proc_macro_attribute]
pub fn grid_fields(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as ItemStruct);
    let str_ident = &ast.ident;
    let grid_type = parse_macro_input!(metadata as Type);

    let dummy: ItemStruct = parse_quote! {
    struct Dummy {
            grid_size: GridSize,
            grid_data: Box<[Box<[#grid_type]>]>,
    }
    };

    if let Fields::Named(ref mut fields) = ast.fields {
        if let Fields::Named(dfields) = dummy.fields {
            for field in dfields.named.into_iter() {
                fields.named.push(field);
            }
        }
    } else {
        panic!("cannot add grid fields to struct without named fields");
    }

    let grid_impl: ItemImpl = parse_quote! {
    impl Grid<#grid_type> for #str_ident {
    }
    };

    let mut tokens = ast.to_token_stream();
    tokens.extend(grid_impl.to_token_stream());
    tokens.into()
}

// TODO: will probably go away
#[proc_macro_derive(Grid)]
pub fn grid(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as ItemStruct);

    // TODO
    //panic!("{:?}", ast);

    /*struct FieldVisitor {}
    impl Visit for FieldVisitor {
    fn visit_fields_named(&mut self, i: &FieldsNamed) {

    }
    }*/

    // Delve down and extract the grid type from the grid_data field
    let mut success = false;
    if let Fields::Named(fields) = ast.fields {
        if let Some(field) = fields
            .named
            .iter()
            .find(|field| field.ident.as_ref().map_or(false, |id| id == "grid_data"))
        {
            if let Type::Path(type_path) = &field.ty {
                //type_path.path.segments.iter()
                success = true;
            }
        }
    }

    if !success {
        panic!("struct does not have a valid 'grid_data' field, try using the 'grid_fields' attribute macro, which needs to come first");
    }

    todo!()
}
