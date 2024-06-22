use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, token, Data, DeriveInput, GenericParam, Result, WhereClause, WherePredicate,
};

use crate::common::{ensure, first_generic_ty, generics_except, is_repr_c};

/// Implements `Borrow`, `BorrowMut`, `From`, `Index`, `IndexMut`, and `Default`.
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

    // impl `Borrow`, `BorrowMut`, and `From` in both directions.
    let convert_impls = quote! {
        impl #impl_generics ::core::borrow::Borrow<#name #ty_generics>
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            #where_clause
        {
            fn borrow(&self) -> &#name #ty_generics {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::BorrowMut<#name #ty_generics>
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            #where_clause
        {
            fn borrow_mut(&mut self) -> &mut #name #ty_generics {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::Borrow<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn borrow(&self)
                -> &[#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::BorrowMut<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn borrow_mut(&mut self)
                -> &mut [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::convert::From<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn from(
                value: [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            ) -> Self {
                debug_assert_eq!(
                    ::core::mem::size_of::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
                    >(),
                    ::core::mem::size_of::<#name #ty_generics>()
                );
                // Need ManuallyDrop so that `value` is not dropped by this function.
                let value = ::core::mem::ManuallyDrop::new(value);
                // Copy the bit pattern. The original value is no longer safe to use.
                unsafe { ::core::mem::transmute_copy(&value) }
            }
        }

        impl #impl_generics ::core::convert::From<#name #ty_generics>
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
            #where_clause
        {
            fn from(value: #name #ty_generics) -> Self {
                debug_assert_eq!(
                    ::core::mem::size_of::<#name #ty_generics>(),
                    ::core::mem::size_of::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
                    >()
                );
                // Need ManuallyDrop so that `value` is not dropped by this function.
                let value = ::core::mem::ManuallyDrop::new(value);
                // Copy the bit pattern. The original value is no longer safe to use.
                unsafe { ::core::mem::transmute_copy(&value) }
            }
        }
    };

    // Unwrap the where clause so we can append to it.
    let where_clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
        where_token: token::Where::default(),
        predicates: Punctuated::new(),
    });

    // Add a generic indexing type to our `impl_generics` for Index and IndexMut.
    let index_ty: GenericParam = parse_quote!(__II);
    let mut all_generics = ast.generics.clone();
    all_generics.params.push(index_ty.clone());
    let (index_generics, _, _) = all_generics.split_for_impl();

    // An empty predicate to constrain any const generics
    let const_pred: WherePredicate =
        parse_quote!([(); ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]:);

    // Where clause for `Index`.
    let index_pred = parse_quote!([#felt_ty]: ::core::ops::Index<#index_ty>);
    let mut index_where_clause = where_clause.clone();
    index_where_clause.predicates.push(index_pred);
    index_where_clause.predicates.push(const_pred.clone());

    // Where clause for `IndexMut`.
    let index_mut_pred = parse_quote!([#felt_ty]: ::core::ops::IndexMut<#index_ty>);
    let mut index_mut_where_clause = where_clause.clone();
    index_mut_where_clause.predicates.push(index_mut_pred);
    index_mut_where_clause.predicates.push(const_pred.clone());

    // Where clause for `Default`.
    let default_pred = parse_quote!(#felt_ty: ::core::default::Default + ::core::marker::Copy);
    let mut default_where_clause = where_clause.clone();
    default_where_clause.predicates.push(default_pred);
    default_where_clause.predicates.push(const_pred.clone());

    Ok(quote! {
        #convert_impls

        impl #index_generics ::core::ops::Index<#index_ty> for #name #ty_generics
            #index_where_clause
        {
            type Output = <[#felt_ty] as ::core::ops::Index<#index_ty>>::Output;

            fn index(&self, index: #index_ty)
                -> &<Self as ::core::ops::Index<#index_ty>>::Output
            {
                let arr = ::core::borrow::Borrow::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
                    >::borrow(self);
                <[#felt_ty] as ::core::ops::Index<#index_ty>>::index(arr, index)
            }
        }

        impl #index_generics ::core::ops::IndexMut<#index_ty> for #name #ty_generics
            #index_mut_where_clause
        {
            fn index_mut(&mut self, index: #index_ty)
                -> &mut <Self as ::core::ops::Index<#index_ty>>::Output
            {
                let arr = ::core::borrow::BorrowMut::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
                    >::borrow_mut(self);
                <[#felt_ty] as ::core::ops::IndexMut<#index_ty>>::index_mut(arr, index)
            }
        }

        impl #impl_generics ::core::default::Default for #name #ty_generics
            #default_where_clause
        {
            fn default() -> Self {
                <Self as ::core::convert::From<
                    [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()]
                >>::from(
                    [
                        <#felt_ty as ::core::default::Default>::default();
                        ::core::mem::size_of::<#name<u8 #(, #rest_generics)*>>()
                    ]
                )
            }
        }
    })
}
