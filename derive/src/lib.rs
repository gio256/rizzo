use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, token, Attribute, Data, DeriveInput, Error, GenericParam, Meta, Result,
    WhereClause,
};

/// Prefixes an error message and generates a `syn::Error` from the message.
macro_rules! span_err {
    ($ast:expr, $msg:literal $(,)?) => {
        ::syn::Error::new_spanned($ast, ::core::concat!("rizzo_derive error: ", $msg))
    };
}
pub(crate) use span_err;

/// Returns early with the given error message.
macro_rules! bail {
    ($ast:expr, $msg:literal $(,)?) => {
        return Err($crate::span_err!($ast, $msg))
    };
}

/// Checks the condition and returns early with the given error message if false.
macro_rules! ensure {
    ($cond:expr, $ast:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::span_err!($ast, $msg));
        }
    };
}

/// Parses the `Meta` of a "repr" attribute and returns true if one of the
/// elements is "C".
fn is_meta_c(outer: &Meta) -> bool {
    if let Meta::List(inner) = outer {
        let parsed: Punctuated<Meta, token::Comma> = inner
            .parse_args_with(Punctuated::parse_terminated)
            .unwrap_or_default();
        parsed.iter().any(|meta| meta.path().is_ident("C"))
    } else {
        false
    }
}

/// Returns true if `#[repr(C)]` is contained in the attributes.
fn is_repr_c<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> bool {
    attrs
        .into_iter()
        .any(|attr| attr.path().is_ident("repr") && is_meta_c(&attr.meta))
}

fn try_derive_columns(ast: DeriveInput) -> Result<proc_macro2::TokenStream> {
    // Make sure we're working with a struct.
    let is_struct = matches!(ast.data, Data::Struct(_));
    ensure!(is_struct, &ast, "expected `struct`");

    // Safety: check that the struct is `#[repr(C)]`
    let repr_c = is_repr_c(&ast.attrs);
    ensure!(repr_c, &ast, "column struct must be `#[repr(C)]`");

    // let params = &ast.generics.params;
    // ensure!(params.len() == 1, &ast, "expected a single generic type argument");

    // let felt_ty = match params.first() {
    //     Some(GenericParam::Type(ty)) => &ty.ident,
    //     _ => bail!(&ast, "expected a generic type argument")
    // };

    // The generic type parameter expected to represent a field element.
    let felt_ty = &ast
        .generics
        .type_params()
        .next()
        .ok_or_else(|| span_err!(&ast, "expected at least one generic type argument"))?
        .ident;

    // All additional generic parameters
    let other_generics: Vec<_> = ast
        .generics
        .params
        .iter()
        .filter_map(|generic| match generic {
            GenericParam::Type(ty) if ty.ident != *felt_ty => Some(&ty.ident),
            GenericParam::Const(c) => Some(&c.ident),
            _ => None,
        })
        .collect();

    // Split generics which include both `felt_ty` and `other_generics`.
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // The name of our struct.
    let name = &ast.ident;

    // impl Borrow, BorrowMut, and From in both directions
    let convert_impls = quote! {
        impl #impl_generics ::core::borrow::Borrow<#name #ty_generics>
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            #where_clause
        {
            fn borrow(&self) -> &#name #ty_generics {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::BorrowMut<#name #ty_generics>
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            #where_clause
        {
            fn borrow_mut(&mut self) -> &mut #name #ty_generics {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::Borrow<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn borrow(&self)
                -> &[#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::borrow::BorrowMut<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn borrow_mut(&mut self)
                -> &mut [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            {
                unsafe { ::core::mem::transmute(self) }
            }
        }

        impl #impl_generics ::core::convert::From<
            [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
        >
            for #name #ty_generics
            #where_clause
        {
            fn from(
                value: [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            ) -> Self {
                debug_assert_eq!(
                    ::core::mem::size_of::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
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
            for [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
            #where_clause
        {
            fn from(value: #name #ty_generics) -> Self {
                debug_assert_eq!(
                    ::core::mem::size_of::<#name #ty_generics>(),
                    ::core::mem::size_of::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
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

    // Where clause for Index.
    let index_pred = parse_quote!([#felt_ty]: ::core::ops::Index<#index_ty>);
    let mut index_where_clause = where_clause.clone();
    index_where_clause.predicates.push(index_pred);

    // Where clause for IndexMut.
    let index_mut_pred = parse_quote!([#felt_ty]: ::core::ops::IndexMut<#index_ty>);
    let mut index_mut_where_clause = where_clause.clone();
    index_mut_where_clause.predicates.push(index_mut_pred);

    // Where clause for Default.
    let default_pred = parse_quote!(#felt_ty: ::core::default::Default);
    let mut default_where_clause = where_clause;
    default_where_clause.predicates.push(default_pred);

    Ok(quote! {
        #convert_impls

        impl #index_generics ::core::ops::Index<#index_ty> for #name #ty_generics
            #index_where_clause
        {
            type Output = <[#felt_ty] as ::core::ops::Index<#index_ty>>::Output;

            fn index(&self, index: #index_ty) -> &<Self as ::core::ops::Index<#index_ty>>::Output {
                let arr = ::core::borrow::Borrow::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
                    >::borrow(self);
                <[#felt_ty] as ::core::ops::Index<#index_ty>>::index(arr, index)
            }
        }

        impl #index_generics ::core::ops::IndexMut<#index_ty> for #name #ty_generics
            #index_mut_where_clause
        {
            fn index_mut(&mut self, index: #index_ty) -> &mut <Self as ::core::ops::Index<#index_ty>>::Output {
                let arr = ::core::borrow::BorrowMut::<
                        [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
                    >::borrow_mut(self);
                <[#felt_ty] as ::core::ops::IndexMut<#index_ty>>::index_mut(arr, index)
            }
        }

        impl #impl_generics ::core::default::Default for #name #ty_generics
            #default_where_clause
        {
            fn default() -> Self {
                <Self as ::core::convert::From<
                    [#felt_ty; ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()]
                >>::from(
                    [
                        <#felt_ty as ::core::default::Default>::default();
                        ::core::mem::size_of::<#name<u8 #(, #other_generics)*>>()
                    ]
                )
            }
        }
    })
}

#[proc_macro_derive(Columns)]
pub fn derive_columns(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    try_derive_columns(ast)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
