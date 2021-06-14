//! # `CellMap` implementation

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::{marker::PhantomData, ops::Index};

use nalgebra::Vector2;
use ndarray::Array2;
use serde::{Deserialize, Serialize};

use crate::{
    extensions::ToShape,
    iterators::{
        layerers::Many,
        slicers::{Cells, Windows},
        CellMapIter, CellMapIterMut,
    },
    CellMapError, Layer,
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

    /// Returns an iterator over each cell in all layers of the map.
    pub fn iter(&self) -> CellMapIter<'_, L, T, Many<L>, Cells> {
        CellMapIter::<'_, L, T, Many<L>, Cells>::new_cells(self)
    }

    /// Returns a mutable iterator over each cell in all layers of the map.
    pub fn iter_mut(&mut self) -> CellMapIterMut<'_, L, T, Many<L>, Cells> {
        CellMapIterMut::<'_, L, T, Many<L>, Cells>::new_cells(self)
    }

    /// Returns an iterator over windows of cells in the map.
    ///
    /// The `semi_width` is half the size of the window in the x and y axes, not including
    /// the central cell. E.g. to have a window which is in total 5x5, the `semi_window_size` needs
    /// to be `Vector2::new(2, 2)`.
    pub fn window_iter(
        &self,
        semi_width: Vector2<usize>,
    ) -> Result<CellMapIter<'_, L, T, Many<L>, Windows>, CellMapError> {
        CellMapIter::<'_, L, T, Many<L>, Windows>::new_windows(self, semi_width)
    }

    /// Returns a mutable iterator over windows of cells in the map.
    ///
    /// The `semi_width` is half the size of the window in the x and y axes, not including
    /// the central cell. E.g. to have a window which is in total 5x5, the `semi_window_size` needs
    /// to be `Vector2::new(2, 2)`.
    pub fn window_iter_mut(
        &mut self,
        semi_width: Vector2<usize>,
    ) -> Result<CellMapIterMut<'_, L, T, Many<L>, Windows>, CellMapError> {
        CellMapIterMut::<'_, L, T, Many<L>, Windows>::new_windows(self, semi_width)
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

impl<L, T> Index<L> for CellMap<L, T>
where
    L: Layer,
{
    type Output = Array2<T>;

    fn index(&self, index: L) -> &Self::Output {
        &self.data[index.to_index()]
    }
}
