//! # `CellMap` implementation

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::{collections::HashMap, hash::Hash};

use ndarray::Array2;
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Marker trait which defines the type of the layer index.
///
/// Layer indices should be `enum`s which implement the [`Layer`] trait. By limiting indices to
/// `enums` we can guarentee that all possible values of layer exist, and therefore do not have to
/// provide run time checking that the layer exists within the map.
///
/// This is enforced by the use of the `#[derive(Layer)]` macro, which can only be implemented on
/// `enums`.
///
/// # Safety
///
/// Do not manually implement this trait for non-enum types, as [`CellMap`] will be unable to
/// guarentee that the layer you're attempting to access will be present in the map.
///
/// # Example
/// ```
/// use cell_map::Layer;
///
/// #[derive(Layer, PartialEq, Eq, Hash)]
/// enum MyLayer {
///     Height,
///     Gradient
/// }
/// ```
pub trait Layer: Eq + Hash {}

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides a many-layer 2D map of cellular data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMap<L, T>
where
    L: Layer,
{
    data: HashMap<L, Array2<T>>,
    // cell_size: [f64; 2],
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L, T> CellMap<L, T>
where
    L: Layer,
{
    pub fn empty() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}
