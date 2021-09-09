extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(CharGridDebug)]
pub fn char_grid_debug(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = ast.ident;
    println!("Gaggles {}", name);
    quote!({
    /*impl Debug for #name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.out_fmt(f)
            }
    }*/
    impl #name {
        fn tester(&self) {
        panic!("WTF");
        }
    }
    })
    .into()
}
