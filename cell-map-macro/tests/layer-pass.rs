//! Test that the Layer trait can be derived for enums

use cell_map_macro::Layer;

#[derive(Layer, PartialEq, Eq, Hash)]
pub enum MyLayer {
    Height,
    Gradient,
}
