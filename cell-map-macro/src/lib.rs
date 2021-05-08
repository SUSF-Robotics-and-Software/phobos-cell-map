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
    let variants = match input.data {
        syn::Data::Enum(e) => e.variants,
        _ => panic!("Layer can only be derived on enums"),
    };

    // Get the type name
    let name = &input.ident;

    // Map the varients into the match patterns we need for the index function
    let var_index_patterns = variants.iter().enumerate().map(|(i, v)| {
        let var_name = &v.ident;

        quote! {
            #name::#var_name => #i
        }
    });

    let num_variants = variants.len();

    let impled = quote! {
        impl ::cell_map::Layer for #name {
            const NUM_LAYERS: usize = #num_variants;

            fn index(&self) -> usize {
                match self {
                    #(#var_index_patterns),*
                }
            }
        }
    };

    impled.into()
}
