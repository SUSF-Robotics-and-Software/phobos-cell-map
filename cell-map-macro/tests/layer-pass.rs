//! Test that the Layer trait can be derived for enums

use cell_map::Layer;

#[derive(Layer, Clone)]
pub enum MyLayer {
    Height,
    Gradient,
}
