//! Tests that layer cannot be implemented on structs

use cell_map_macro::Layer;

#[derive(Layer)]
struct NotALayer;
