//! # `cell-map`: many-layer 2D cellular maps
//!
//! This crate provides the `CellMap` type, a 2D map with many layers comprised of
//! cells that can store arbitrary data. It is based on
//! [ANYbotics/grid_map](https://github.com/ANYbotics/grid_map), a C++ ROS package
//! which provides the same type of data structre.
//!
//! `CellMap` uses `ndarray::Array2` to store its data in an efficient and
//! scalable format. It also uses `nalgebra` types for expressing vectors and
//! points.
//!
//! ## Getting Started
//!
//! ### Layers
//!
//! Each layer of the cell map is represented by its own `ndarray::Array2` array.
//! The map indexes each layer by an enum implementing the `Layer` trait. A derive
//! macro is provided to simplify this, for example:
//!
//! ```rust
//! use cell_map::Layer;
//!
//! #[derive(Layer, Clone, Debug)]
//! enum MyLayer {
//!     Height,
//!     Gradient,
//!     Roughness
//! }
//! ```
//!
//! The `Layer` trait is required to be `Clone`, and is recommended to be `Debug`.
//!
//! ### Creating a `CellMap`
//!
//! To create a new map:
//!
//! ```rust
//! use cell_map::{CellMap, CellMapParams};
//! use nalgebra::Vector2;
//!
//! # #[derive(Layer, Clone, Debug)]
//! # enum MyLayer {
//! #     Height,
//! #     Gradient,
//! #     Roughness
//! # }
//! // Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0).
//! let my_map = CellMap::<MyLayer, f64>::new_from_elem(
//!     CellMapParams {
//!         cell_size: Vector2::new(1.0, 1.0),
//!         num_cells: Vector2::new(5, 5),
//!         centre: Vector2::new(0.0, 0.0),
//!     },
//!     1.0,
//! );
//! ```
//!
//! ### Iterating Over Cells
//!
//! `CellMap` provides the following iteration types over its cells:
//!   - `CellIter` - iterate each cell in the map
//!   - `WindowIter` - iterate a window through the map
//!
//! All iterators also provide a mutable variant, and more iterators are planned
//! in the future!
//!
//! You can modify iterators so they produce `Layered` or `Indexed` iterators as
//! well.
//!
//! ```rust
//! # use cell_map::{CellMap, CellMapParams, Layer};
//! # use nalgebra::Vector2;
//! #
//! # #[derive(Layer, Clone, Debug)]
//! # enum MyLayer {
//! #     Height,
//! #     Gradient,
//! #     Roughness
//! # }
//! #
//! # // Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0).
//! # let mut my_map = CellMap::<MyLayer, f64>::new_from_elem(
//! #     CellMapParams {
//! #         cell_size: Vector2::new(1.0, 1.0),
//! #         num_cells: Vector2::new(5, 5),
//! #         centre: Vector2::new(0.0, 0.0),
//! #     },
//! #     1.0,
//! # );
//! // Check all the cells in our map are 1, this will be true
//! assert!(my_map.iter().all(|v| v == 1.0));
//!
//! // Use a window iterator to change all cells not on the border of the map to 2
//! my_map.window_iter_mut(Vector2::new(1, 1)).for_each(|mut v| {
//!     v[(1, 1)] = 2.0;
//! });
//!
//! // Overwrite all values on the Roughness layer to be zero
//! my_map.iter_mut().layer(MyLayer::Roughness).for_each(|v| *v = 0.0);
//!
//! // Check that our map is how we expect it
//! for ((layer, cell), value) in my_map.iter().indexed() {
//!     if let MyLayer::Roughness = layer {
//!         assert_eq!(value, 0.0);
//!     }
//!     else if cell.x == 0 || cell.x == 4 || cell.y == 0 || cell.y == 4 {
//!         assert_eq!(value, 1.0);
//!     }
//!     else {
//!         assert_eq!(value, 2.0);
//!     }
//! }
//! ```

#![warn(missing_docs)]

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
