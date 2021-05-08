//! # `cell-map`: 2.5D cellular maps
//!
//! This crate provides the [`CellMap`] type, which provides a way of managing 2D maps of cellular
//! data. It is based on [ANYbotics/grid_map](https://github.com/ANYbotics/grid_map) for C++ and
//! ROS, and uses the same conventions for cell indexing and grid map frame orientations.
//!
//! See the [`CellMap`] documentation for more information.

// ------------------------------------------------------------------------------------------------
// MODULES
// ------------------------------------------------------------------------------------------------

mod cell_map;
pub(crate) mod extensions;
pub mod iterators;
mod layer;

// ------------------------------------------------------------------------------------------------
// EXPORTS
// ------------------------------------------------------------------------------------------------

pub use crate::cell_map::{CellMap, CellMapParams};
pub use cell_map_macro::Layer;
pub use layer::Layer;
