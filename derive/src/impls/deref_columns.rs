use quote::quote;
use syn::{Data, DeriveInput, Result};

use crate::common::{ensure, first_generic_ty, generics_except, is_repr_c};

/// Implements `Deref` and `DerefMut`.
pub(crate) fn try_derive(ast: DeriveInput) -> Result<proc_macro2::TokenStream> {
    // Make sure we're working with a struct.
    let is_struct = matches!(ast.data, Data::Struct(_));
    ensure!(is_struct, &ast, "expected `struct`");

    // Safety: check that the struct is `#[repr(C)]`
    let repr_c = is_repr_c(&ast.attrs);
    ensure!(repr_c, &ast, "column struct must be `#[repr(C)]`");

    // The first generic type parameter is expected to represent a field element.
    let felt_ty = first_generic_ty(&ast)?;

    // All additional generic parameters.
    let rest_generics = generics_except(&ast, felt_ty);

    // Split all the generics, including both `felt_ty` and `rest_generics`.
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // The name of our struct.
    let name = &ast.ident;

    Ok(quote! {
        impl #impl_generics ::core::ops::Deref for #name #ty_generics
            #where_clause
        {
            type Target = [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()];

            fn deref(&self) -> &Self::Target {
                unsafe { core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #name #ty_generics
            #where_clause
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { core::mem::transmute(self) }
            }

        }
    })
}
