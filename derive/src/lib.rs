pub(crate) mod common;
mod impls;

use impls::{columns, deref_columns};

#[proc_macro_derive(Columns)]
pub fn derive_columns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    columns::try_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DerefColumns)]
pub fn derive_deref_columns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    deref_columns::try_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
