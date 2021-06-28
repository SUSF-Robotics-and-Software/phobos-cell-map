//! # `CellMap` implementation

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    usize,
};

use nalgebra::{Affine2, Point2, Vector2};
use ndarray::Array2;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    cell_map_file::CellMapFile,
    extensions::ToShape,
    iterators::{
        layerers::Many,
        slicers::{Cells, Line, Windows},
        CellMapIter, CellMapIterMut,
    },
    map_metadata::CellMapMetadata,
    CellMapError, Layer,
};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides a many-layer 2D map of cellular data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "CellMapFile<L, T>",
    into = "CellMapFile<L, T>",
    bound = "T: Clone + Serialize + DeserializeOwned, L: Serialize + DeserializeOwned"
)]
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

    /// Metadata associated with this map.
    pub(crate) metadata: CellMapMetadata,

    /// The original parameters supplied to `CellMap::new()`.
    pub(crate) params: CellMapParams,

    layer_type: PhantomData<L>,
}

/// Contains parameters required to construct a [`CellMap`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CellMapParams {
    /// The size (resolution) of each cell in the map, in parent frame coordinates.
    ///
    /// # Default
    ///
    /// The default value is `[1.0, 1.0]`.
    pub cell_size: Vector2<f64>,

    /// The number of cells in the `x` and `y` directions.
    ///
    /// # Default
    ///
    /// The default value is `[0, 0]`.
    pub num_cells: Vector2<usize>,

    /// The rotation of the map's Z axis about the parent Z axis in radians.
    ///
    /// # Default
    ///
    /// The default value is `0.0`.
    pub rotation_in_parent_rad: f64,

    /// The position of the origin of the map in the parent frame, in parent frame units.
    ///
    /// # Default
    ///
    /// The default value is `[0.0, 0.0]`.
    pub position_in_parent: Vector2<f64>,

    /// The precision to use when determining cell boundaries.
    ///
    /// This precision factor allows us to account for times when a cell position should fit into a
    /// particular cell index, but due to floating point rounding does not. For example take a map
    /// with a `cell_size = [0.1, 0.1]`, the cell index of the position `[0.7, 0.1]` should be `[7,
    /// 1`], however the positions floating point index would be calculated as `[6.999999999999998,
    /// 0.9999999999999999]`, which if `floor()`ed to fit into a `usize` would give the incorrect
    /// index `[6, 0]`.
    ///
    /// When calculating cell index we therefore `floor` the floating point index unless it is
    /// within `cell_size * cell_boundary_precision`, in which case we round up to the next cell.
    /// Mutliplying by `cell_size` allows this value to be independent of the scale of the map.
    ///
    /// # Default
    ///
    /// The default value is `1e-10`.
    pub cell_boundary_precision: f64,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L, T> CellMap<L, T>
where
    L: Layer,
{
    /// Creates a new map from the given data.
    ///
    /// If data is the wrong shape or has the wrong number of layers this function will return an
    /// error.
    pub fn new_from_data(
        params: CellMapParams,
        data: Vec<Array2<T>>,
    ) -> Result<Self, CellMapError> {
        if data.len() != L::NUM_LAYERS {
            return Err(CellMapError::WrongNumberOfLayers(L::NUM_LAYERS, data.len()));
        }

        if !data.is_empty() {
            let layer_cells = Vector2::new(data[0].shape()[0], data[0].shape()[1]);

            if layer_cells != params.num_cells {
                return Err(CellMapError::LayerWrongShape(layer_cells, params.num_cells));
            }
        }

        Ok(Self {
            data,
            metadata: params.into(),
            params,
            layer_type: PhantomData,
        })
    }

    /// Returns the size of the cells in the map.
    pub fn cell_size(&self) -> Vector2<f64> {
        self.metadata.cell_size
    }

    /// Returns the number of cells in each direction of the map.
    pub fn num_cells(&self) -> Vector2<usize> {
        self.metadata.num_cells
    }

    /// Returns the parameters used to build this map.
    pub fn params(&self) -> CellMapParams {
        self.params
    }

    /// Gets the [`nalgebra::Affine2<f64>`] transformation between the map frame and the parent
    /// frame.
    pub fn to_parent(&self) -> Affine2<f64> {
        self.metadata.to_parent
    }

    /// Returns whether or not the given index is inside the map.
    pub fn is_in_map(&self, index: Point2<usize>) -> bool {
        self.metadata.is_in_map(index)
    }

    /// Get a reference to the value at the given layer and index. Returns `None` if the index is
    /// outside the bounds of the map.
    pub fn get(&self, layer: L, index: Point2<usize>) -> Option<&T> {
        if self.is_in_map(index) {
            Some(&self[(layer, index)])
        } else {
            None
        }
    }

    /// Get a reference to the value at the given layer and index, without checking the bounds of
    /// the map.
    ///
    /// # Safety
    ///
    /// This function will panic if `index` is outside the map.
    pub unsafe fn get_unchecked(&self, layer: L, index: Point2<usize>) -> &T {
        &self[(layer, index)]
    }

    /// Get a mutable reference to the value at the given layer and index. Returns `None` if the
    /// index is outside the bounds of the map.
    pub fn get_mut(&mut self, layer: L, index: Point2<usize>) -> Option<&mut T> {
        if self.is_in_map(index) {
            Some(&mut self[(layer, index)])
        } else {
            None
        }
    }

    /// Get a mutable reference to the value at the given layer and index, without checking the
    /// bounds of the map.
    ///
    /// # Safety
    ///
    /// This function will panic if `index` is outside the map.
    pub unsafe fn get_mut_unchecked(&mut self, layer: L, index: Point2<usize>) -> &mut T {
        &mut self[(layer, index)]
    }

    /// Returns the position in the parent frame of the centre of the given cell index.
    ///
    /// Returns `None` if the given `index` is not inside the map.
    pub fn position(&self, index: Point2<usize>) -> Option<Point2<f64>> {
        self.metadata.position(index)
    }

    /// Returns the position in the parent frame of the centre of the given cell index, without
    /// checking that the `index` is inside the map.
    ///
    /// # Safety
    ///
    /// This method won't panic if `index` is outside the map, but it's result can't be guaranteed
    /// to be a position in the map.
    pub fn position_unchecked(&self, index: Point2<usize>) -> Point2<f64> {
        self.metadata.position_unchecked(index)
    }

    /// Get the cell index of the given poisition.
    ///
    /// Returns `None` if the given `position` is not inside the map.
    pub fn index(&self, position: Point2<f64>) -> Option<Point2<usize>> {
        self.metadata.index(position)
    }

    /// Get the cell index of the given poisition, without checking that the position is inside the
    /// map.
    ///
    /// # Safety
    ///
    /// This function will not panic if `position` is outside the map, but use of the result to
    /// index into the map is not guaranteed to be safe. It is possible for this function to return
    /// a negative index value, which would indicate that the cell is outside the map.
    pub unsafe fn index_unchecked(&self, position: Point2<f64>) -> Point2<isize> {
        self.metadata.index_unchecked(position)
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

    /// Returns an iterator over cells along the line joining `start_position` and
    /// `end_position`, which are expressed as positions in the map's parent frame.
    pub fn line_iter(
        &self,
        start_position: Point2<f64>,
        end_position: Point2<f64>,
    ) -> Result<CellMapIter<'_, L, T, Many<L>, Line>, CellMapError> {
        CellMapIter::<'_, L, T, Many<L>, Line>::new_line(self, start_position, end_position)
    }

    /// Returns a mutable iterator over cells along the line joining `start_position` and
    /// `end_position`, which are expressed as positions in the map's parent frame.
    pub fn line_iter_mut(
        &mut self,
        start_position: Point2<f64>,
        end_position: Point2<f64>,
    ) -> Result<CellMapIterMut<'_, L, T, Many<L>, Line>, CellMapError> {
        CellMapIterMut::<'_, L, T, Many<L>, Line>::new_line(self, start_position, end_position)
    }
}

impl<L, T> CellMap<L, T>
where
    L: Layer + Serialize,
    T: Clone + Serialize,
{
    /// Builds a new [`CellMapFile`] from the given map, which can be serialised or deserialised
    /// using serde.
    pub fn to_cell_map_file(&self) -> CellMapFile<L, T> {
        CellMapFile::new(self)
    }

    /// Writes the map to the given path as a JSON file.
    #[cfg(feature = "json")]
    pub fn write_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), CellMapError> {
        let map_file = CellMapFile::new(&self);
        map_file
            .write_json(path)
            .map_err(|e| CellMapError::WriteError(e))
    }
}

impl<L, T> CellMap<L, T>
where
    L: Layer + DeserializeOwned,
    T: DeserializeOwned,
{
    /// Loads a map stored in JSON format at the given path.
    #[cfg(feature = "json")]
    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, CellMapError> {
        let map_file = CellMapFile::from_json(path).map_err(|e| CellMapError::LoadError(e))?;
        map_file.into_cell_map()
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
            metadata: params.into(),
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
            metadata: params.into(),
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

impl<L, T> IndexMut<L> for CellMap<L, T>
where
    L: Layer,
{
    fn index_mut(&mut self, index: L) -> &mut Self::Output {
        &mut self.data[index.to_index()]
    }
}

impl<L, T> Index<(L, Point2<usize>)> for CellMap<L, T>
where
    L: Layer,
{
    type Output = T;

    fn index(&self, index: (L, Point2<usize>)) -> &Self::Output {
        &self[index.0][(index.1.y, index.1.x)]
    }
}

impl<L, T> IndexMut<(L, Point2<usize>)> for CellMap<L, T>
where
    L: Layer,
{
    fn index_mut(&mut self, index: (L, Point2<usize>)) -> &mut Self::Output {
        &mut self[index.0][(index.1.y, index.1.x)]
    }
}

impl Default for CellMapParams {
    fn default() -> Self {
        Self {
            cell_size: Vector2::new(1.0, 1.0),
            num_cells: Vector2::zeros(),
            cell_boundary_precision: 1e-10,
            rotation_in_parent_rad: 0.0,
            position_in_parent: Vector2::zeros(),
        }
    }
}
