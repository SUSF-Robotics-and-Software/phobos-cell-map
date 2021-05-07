//! # `cell-map` Macros
//!
//! This crate provides implementations of macros for the [`cell_map`](todo) crate.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// ------------------------------------------------------------------------------------------------
// DERIVES
// ------------------------------------------------------------------------------------------------

#[proc_macro_derive(Layer)]
pub fn derive_layer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Check input is an enum
    match input.data {
        syn::Data::Enum(_) => (),
        _ => panic!("Layer can only be derived on enums"),
    }

    // Get the type name
    let name = input.ident;

    let impled = quote! {
        impl ::cell_map::Layer for #name {}
    };

    impled.into()
}
