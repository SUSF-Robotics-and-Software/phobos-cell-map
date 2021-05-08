//! # `CellMap` implementation

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use nalgebra::Vector2;
use ndarray::Array2;
use serde::{Deserialize, Serialize};

use crate::{
    extensions::ToShape,
    iterators::{CellIter, CellIterMut},
    Layer,
};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides a many-layer 2D map of cellular data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMap<L, T>
where
    L: Layer,
{
    /// Stores each layer in the map as an [`ndarray::Array2<T>`].
    ///
    /// TODO:
    /// When constgenerics is stabilised would be good to make this an array of `L::NUM_LAYERS`, to
    /// avoid the vec allocation.
    pub(crate) data: Vec<Array2<T>>,

    pub(crate) params: CellMapParams,

    layer_type: PhantomData<L>,
}

/// Contains parameters required to construct a [`CellMap`]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellMapParams {
    /// The size (resolution) of each cell in the map, in both the `x` and `y` directions.
    pub cell_size: Vector2<f64>,

    /// The number of cells in the `x` and `y` directions.
    pub num_cells: Vector2<usize>,

    /// The position of the centre of the grid map.
    pub centre: Vector2<f64>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L, T> CellMap<L, T>
where
    L: Layer,
{
    /// Returns the size of the cells in the map.
    pub fn cell_size(&self) -> Vector2<f64> {
        self.params.cell_size.clone()
    }

    /// Returns the number of cells in each direction of the map.
    pub fn num_cells(&self) -> Vector2<usize> {
        self.params.num_cells.clone()
    }

    /// Returns a mutable iterator over each cell in each layer of the map.
    pub fn iter_mut(&mut self) -> CellIterMut<L, T> {
        CellIterMut {
            layer_limits: None,
            limits_idx: None,
            index: (0, 0, 0),
            map: self,
        }
    }
}

impl<L, T> CellMap<L, T>
where
    L: Layer,
    T: Clone,
{
    /// Creates a new [`CellMap`] from the given params, filling each cell with `elem`.
    pub fn new_from_elem(params: CellMapParams, elem: T) -> Self {
        let data = vec![Array2::from_elem(params.num_cells.to_shape(), elem); L::NUM_LAYERS];

        Self {
            data,
            params,
            layer_type: PhantomData,
        }
    }

    /// Produces an iterator of owned items over each cell in each layer of `self`.
    pub fn iter(&self) -> CellIter<L, T> {
        CellIter {
            layer_limits: None,
            limits_idx: None,
            index: (0, 0, 0),
            map: &self,
        }
    }
}

impl<L, T> CellMap<L, T>
where
    L: Layer,
    T: Default + Clone,
{
    /// Creates a new [`CellMap`] from the given params, filling each cell with `T::default()`.
    pub fn new(params: CellMapParams) -> Self {
        let data =
            vec![Array2::from_elem(params.num_cells.to_shape(), T::default()); L::NUM_LAYERS];

        Self {
            data,
            params,
            layer_type: PhantomData,
        }
    }
}
