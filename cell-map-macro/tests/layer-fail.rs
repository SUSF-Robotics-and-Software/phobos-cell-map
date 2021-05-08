//! Tests that layer cannot be implemented on structs

use cell_map::Layer;

#[derive(Layer)]
struct NotALayer;
