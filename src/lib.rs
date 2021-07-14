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
//! use cell_map::{CellMap, CellMapParams, Layer, Bounds};
//! use nalgebra::Vector2;
//!
//! # #[derive(Layer, Clone, Debug)]
//! # enum MyLayer {
//! #     Height,
//! #     Gradient,
//! #     Roughness
//! # }
//! // Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0), with
//! // all elements initialised to 1.0. The `cell_bounds` parameter allows you to choose how many
//! // cells are in the map by giving the min and max cell indexes (including negative indexes).
//! let my_map = CellMap::<MyLayer, f64>::new_from_elem(
//!     CellMapParams {
//!         cell_size: Vector2::new(1.0, 1.0),
//!         cell_bounds: Bounds::new((0, 5), (0, 5)).unwrap(),
//!         ..Default::default()
//!     },
//!     1.0,
//! );
//! ```
//!
//! ### Iterating Over Cells
//!
//! [`CellMap`] provides methods to produce iterators over its data:
//!   - [`CellMap::iter()`] gives an iterator over all cells in every layer of the map
//!   - [`CellMap::window_iter()`] gives an iterator over rectangular windows into the map
//!   - [`CellMap::line_iter()`] gives an iterator of cells between two world points
//!
//! All iterators also provide a mutable variant, and more iterators are planned
//! in the future!
//!
//! You can modify iterators so they produce their indexes, as well as controlling which layers the
//! data comes from. See [`iterators`] for more information.
//!
//! ```rust
//! # use cell_map::{CellMap, CellMapParams, Layer, Bounds};
//! # use nalgebra::Vector2;
//! #
//! # #[derive(Layer, Clone, Debug)]
//! # enum MyLayer {
//! #     Height,
//! #     Gradient,
//! #     Roughness
//! # }
//! #
//! # // Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0), with
//! # // all elements initialised to 1.0.
//! # let mut my_map = CellMap::<MyLayer, f64>::new_from_elem(
//! #     CellMapParams {
//! #         cell_size: Vector2::new(1.0, 1.0),
//! #         cell_bounds: Bounds::new((0, 5), (0, 5)).unwrap(),
//! #         ..Default::default()
//! #     },
//! #     1.0,
//! # );
//! // Check all the cells in our map are 1, this will be true
//! assert!(my_map.iter().all(|&v| v == 1.0));
//!
//! // Use a window iterator to change all cells not on the border of the map to 2
//! my_map.window_iter_mut(Vector2::new(1, 1)).unwrap().for_each(|mut v| {
//!     v[(1, 1)] = 2.0;
//! });
//!
//! // Overwrite all values on the Roughness layer to be zero
//! my_map.iter_mut().layer(MyLayer::Roughness).for_each(|v| *v = 0.0);
//!
//! // Check that our map is how we expect it
//! for ((layer, cell), &value) in my_map.iter().indexed() {
//!     if matches!(layer, MyLayer::Roughness) {
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
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

// ------------------------------------------------------------------------------------------------
// MODULES
// ------------------------------------------------------------------------------------------------

#[macro_use]
mod macros;

pub(crate) mod cell_map;
pub mod cell_map_file;
pub mod error;
pub(crate) mod extensions;
pub mod iterators;
mod layer;
mod map_metadata;
#[cfg(test)]
mod tests;

// ------------------------------------------------------------------------------------------------
// EXPORTS
// ------------------------------------------------------------------------------------------------

pub use crate::cell_map::{Bounds, CellMap, CellMapParams};
pub use cell_map_macro::Layer;
pub use error::Error;
pub use layer::Layer;

// ------------------------------------------------------------------------------------------------
// USEFUL TEST UTILITIES
// ------------------------------------------------------------------------------------------------

#[cfg(feature = "debug_maps")]
use serde::Serialize;

/// Writes the given map to the given location, prepending "_debug_" to the name.
#[cfg(feature = "debug_maps")]
pub fn write_debug_map<L: Layer + Serialize, T: Serialize + Clone>(
    map: &CellMap<L, T>,
    name: &str,
) {
    #[cfg(feature = "debug_maps")]
    {
        let file_name = &format!("_debug_{}_map.json", name);
        map.write_json(file_name)
            .expect("Failed to write debug map!");
    }
}
#[cfg(test)]
#[macro_use]
pub(crate) mod test_utils {

    use serde::Serialize;

    use crate::Layer;

    #[derive(Clone, Copy, Debug, Serialize)]
    #[allow(dead_code)]
    pub enum TestLayers {
        Layer0,
        Layer1,
        Layer2,
    }

    // Have to do a manual impl because the derive doesn't like working inside this crate, for some
    // reason
    impl Layer for TestLayers {
        const NUM_LAYERS: usize = 3;
        const FIRST: Self = Self::Layer0;
        fn to_index(&self) -> usize {
            match self {
                Self::Layer0 => 0,
                Self::Layer1 => 1,
                Self::Layer2 => 2,
            }
        }

        fn from_index(index: usize) -> Self {
            match index {
                0 => Self::Layer0,
                1 => Self::Layer1,
                2 => Self::Layer2,
                _ => panic!(
                    "Got a layer index of {} but there are only {} layers",
                    index,
                    Self::NUM_LAYERS
                ),
            }
        }

        fn all() -> Vec<Self> {
            vec![Self::Layer0, Self::Layer1, Self::Layer2]
        }
    }
}
