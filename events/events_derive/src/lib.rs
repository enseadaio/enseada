use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Event)]
pub fn events_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse TokenStream when deriving trait events::Event");
    impl_events_macro(&ast)
}

fn impl_events_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Event for #name {}
    };
    gen.into()
}