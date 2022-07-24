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
use ndarray::{s, Array2};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    cell_map_file::CellMapFile,
    extensions::Point2Ext,
    iterators::{
        layerers::Many,
        slicers::{Cells, Line, Windows},
        CellMapIter, CellMapIterMut,
    },
    map_metadata::CellMapMetadata,
    Error, Layer,
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
    pub metadata: CellMapMetadata,

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
    /// The default value is [`Bounds::empty()`].
    pub cell_bounds: Bounds,

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

/// Rectangular bounds describing the number of cells in each direction of the map.
///
/// These bounds are a half-open range, i.e. satisfied in the ranges:
///  - $x_0 <= x < x_1$
///  - $y_0 <= y < y_1$
// NOTE: Range isn't uses since it's not Copy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bounds {
    /// The bounds on the x axis, in the format (min, max),
    pub x: (isize, isize),

    /// The bounds on the y axis, in the format (min, max),
    pub y: (isize, isize),
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
    pub fn new_from_data(params: CellMapParams, data: Vec<Array2<T>>) -> Result<Self, Error> {
        if data.len() != L::NUM_LAYERS {
            return Err(Error::WrongNumberOfLayers(L::NUM_LAYERS, data.len()));
        }

        if !data.is_empty() {
            let layer_cells = (data[0].shape()[0], data[0].shape()[1]);

            if layer_cells != params.cell_bounds.get_shape() {
                return Err(Error::LayerWrongShape(
                    layer_cells,
                    params.cell_bounds.get_shape(),
                ));
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

    /// Returns the bounds of this map
    pub fn cell_bounds(&self) -> Bounds {
        self.metadata.cell_bounds
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

    /// Moves this map relative to a new position and rotation relative to the parent frame.
    ///
    /// **Note:** This doesn't move the data relative to the map origin, the indexes into the map
    /// remain the same, but the position of each cell in the map will change.
    pub fn move_map(&mut self, position_in_parent: Vector2<f64>, rotation_in_parent_rad: f64) {
        // Recalculate the map's to_parent affine
        self.metadata.to_parent = CellMapMetadata::calc_to_parent(
            position_in_parent,
            rotation_in_parent_rad,
            self.metadata.cell_size,
        );

        // Update the parameter values
        self.params.position_in_parent = position_in_parent;
        self.params.rotation_in_parent_rad = rotation_in_parent_rad;
    }

    /// Returns whether or not the given index is inside the map.
    pub fn index_in_map(&self, index: Point2<usize>) -> bool {
        self.metadata.is_in_map(index)
    }

    /// Returns whether or not the given parent-relative position is inside the map.
    pub fn position_in_map(&self, position: Point2<f64>) -> bool {
        self.index(position).is_some()
    }

    /// Get a reference to the value at the given layer and index. Returns `None` if the index is
    /// outside the bounds of the map.
    pub fn get(&self, layer: L, index: Point2<usize>) -> Option<&T> {
        if self.index_in_map(index) {
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
        if self.index_in_map(index) {
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

    /// Set the given layer and index in the map to the given value. Returns an [`Error`] if the
    /// index was outside the map.
    pub fn set(&mut self, layer: L, index: Point2<usize>, value: T) -> Result<(), Error> {
        if self.index_in_map(index) {
            self[(layer, index)] = value;
            Ok(())
        } else {
            Err(Error::IndexOutsideMap(index))
        }
    }

    /// Set the given layer and index in the map to the given value, without checking if index is
    /// the map.
    ///
    /// # Safety
    ///
    /// This function will panic if `index` is outside the map
    pub unsafe fn set_unchecked(&mut self, layer: L, index: Point2<usize>, value: T) {
        self[(layer, index)] = value;
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
    ) -> Result<CellMapIter<'_, L, T, Many<L>, Windows>, Error> {
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
    ) -> Result<CellMapIterMut<'_, L, T, Many<L>, Windows>, Error> {
        CellMapIterMut::<'_, L, T, Many<L>, Windows>::new_windows(self, semi_width)
    }

    /// Returns an iterator over cells along the line joining `start_position` and
    /// `end_position`, which are expressed as positions in the map's parent frame.
    pub fn line_iter(
        &self,
        start_position: Point2<f64>,
        end_position: Point2<f64>,
    ) -> Result<CellMapIter<'_, L, T, Many<L>, Line>, Error> {
        CellMapIter::<'_, L, T, Many<L>, Line>::new_line(self, start_position, end_position)
    }

    /// Returns a mutable iterator over cells along the line joining `start_position` and
    /// `end_position`, which are expressed as positions in the map's parent frame.
    pub fn line_iter_mut(
        &mut self,
        start_position: Point2<f64>,
        end_position: Point2<f64>,
    ) -> Result<CellMapIterMut<'_, L, T, Many<L>, Line>, Error> {
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
    pub fn write_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Error> {
        let map_file = CellMapFile::new(&self);
        map_file.write_json(path)
    }
}

impl<L, T> CellMap<L, T>
where
    L: Layer + DeserializeOwned,
    T: DeserializeOwned,
{
    /// Loads a map stored in JSON format at the given path.
    #[cfg(feature = "json")]
    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let map_file = CellMapFile::from_json(path)?;
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
        let data = vec![Array2::from_elem(params.cell_bounds.get_shape(), elem); L::NUM_LAYERS];

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
            vec![Array2::from_elem(params.cell_bounds.get_shape(), T::default()); L::NUM_LAYERS];

        Self {
            data,
            metadata: params.into(),
            params,
            layer_type: PhantomData,
        }
    }

    /// Resizes the map into the new bounds, filling any newly added cells with `T::default()`.
    ///
    /// Any cells that are in the map currently, which would be outside the new map, are removed.
    // NOTE: It doesn't seem possible to resize an ndarray in place, so we have to allocate a new
    // one.
    pub fn resize(&mut self, new_bounds: Bounds) {
        // Allocate new data
        let mut data = vec![Array2::from_elem(new_bounds.get_shape(), T::default()); L::NUM_LAYERS];

        // Get the slice describing the position of the old map inside the new map, based on the
        // bounds. If there's no intersection then we can skip this step
        if let Some(old_in_new) = new_bounds.get_slice_of_other(&self.metadata.cell_bounds) {
            // Get the slice of new relative to old. Unwrap is ok sice we already know there's an
            // intersection.
            let new_in_old = self
                .metadata
                .cell_bounds
                .get_slice_of_other(&new_bounds)
                .unwrap();
            for (new, old) in data.iter_mut().zip(self.data.iter()) {
                new.slice_mut(s![
                    old_in_new.y.0..old_in_new.y.1,
                    old_in_new.x.0..old_in_new.x.1
                ])
                .assign(&old.slice(s![
                    new_in_old.y.0..new_in_old.y.1,
                    new_in_old.x.0..new_in_old.x.1
                ]));
            }
        }

        self.data = data;
        self.metadata.cell_bounds = new_bounds;
        self.params.cell_bounds = new_bounds;
        self.metadata.num_cells = new_bounds.get_num_cells();
    }

    /// Merge `other` into self, resizing `self` so that `other` will be fully included in the map.
    ///
    /// Both maps should belong to the same parent frame, and `other.cell_size <= self.cell_size`.
    /// For `other`s that have larger cells than `self` you should implement your own merge functon
    /// based on this one that could for example use 2D linear interpolation.
    ///
    /// `func` is responsible for actually merging data in both `self` and `other` into a single
    /// new value in `self`. The first argument is the value of the cell in `self`, while the
    /// second argument will be the values from cells in `other` whose centres lie within the cell
    /// in `self`.
    pub fn merge<F: Fn(&T, &[T]) -> T>(&mut self, other: &CellMap<L, T>, func: F) {
        // First get the bounds of `other` wrt `self`, which we have to do by accounting for the
        // potential different alignment of `other` wrt `parent`. We do this by getting the corner
        // points, transforming from `other` to `parent`, then from `parent` to `self`. We have to
        // transform all corner points because rotation may lead to the corners being in different
        // positions than when aligned to `other`.
        let other_bounds = other.cell_bounds();
        let corners_in_other = vec![
            Point2::new(other_bounds.x.0, other_bounds.y.0).cast(),
            Point2::new(other_bounds.x.1, other_bounds.y.0).cast() + Vector2::new(1.0, 0.0),
            Point2::new(other_bounds.x.0, other_bounds.y.1).cast() + Vector2::new(0.0, 1.0),
            Point2::new(other_bounds.x.1, other_bounds.y.1).cast() + Vector2::new(1.0, 1.0),
        ];
        let corners_in_parent: Vec<Point2<f64>> = corners_in_other
            .iter()
            .map(|c| other.to_parent().transform_point(c))
            .collect();
        let other_bl_parent = Point2::new(
            corners_in_parent
                .iter()
                .min_by_key(|c| c.x.floor() as isize)
                .unwrap()
                .x
                .floor(),
            corners_in_parent
                .iter()
                .min_by_key(|c| c.y.floor() as isize)
                .unwrap()
                .y
                .floor(),
        );
        let other_ur_parent = Point2::new(
            corners_in_parent
                .iter()
                .max_by_key(|c| c.x.ceil() as isize)
                .unwrap()
                .x
                .ceil(),
            corners_in_parent
                .iter()
                .max_by_key(|c| c.y.ceil() as isize)
                .unwrap()
                .y
                .ceil(),
        );
        let other_in_self =
            Bounds::from_corner_positions(&self.metadata, other_bl_parent, other_ur_parent);
        let store_offset = Vector2::new(
            other_in_self.x.0.clamp(0, self.num_cells().x as isize) as usize,
            other_in_self.y.0.clamp(0, self.num_cells().y as isize) as usize,
        );

        // Calculate the union of both bounds
        let new_bounds = self.cell_bounds().union(&other_in_self);

        // Resize self
        self.resize(new_bounds);

        // Get the index offset to go from an index into self to the 2D storage array (see store)
        let store_slice_in_new = if let Some(slice) = new_bounds.get_slice_of_other(&other_in_self)
        {
            slice
        } else {
            unreachable!("Other was not inside self's new bounds");
        };

        // For each layer in the map
        for layer in L::all() {
            // Create a new array of size other_in_self, which will hold a copy of all items in
            // other which fall into each cell in self.
            let mut store: Array2<Vec<T>> = Array2::default(other_in_self.get_shape());

            // For each cell in other get its position in parent, convert that to a cell index in
            // self, and add that cell's value to the store
            for ((_, pos), val) in other.iter().layer(layer.clone()).positioned() {
                // The index of pos in self
                if let Some(idx) = self.index(pos) {
                    // Get the index into the store array by subtracting the store offset
                    let store_idx = (idx.cast() - store_offset).map(|e| e as usize);

                    // Mutate the store vector by pushing val into it
                    if let Some(vec) = store.get_mut(store_idx.as_array2_index()) {
                        vec.push(val.clone());
                    } else {
                        unreachable!("Store index {} was invalid", store_idx);
                    }
                } else {
                    // Point was outside the map, this shouldn't happen
                    unreachable!("Point in other ({}) was outside self during merge", pos);
                }
            }

            // Iterate over the store and self, calling the merge function with the value in self
            // and the values in the store
            for (self_val, store_vec) in self.data[layer.to_index()]
                .slice_mut(s![
                    store_slice_in_new.y.0..store_slice_in_new.y.1,
                    store_slice_in_new.x.0..store_slice_in_new.x.1,
                ])
                .iter_mut()
                .zip(store.iter())
            {
                *self_val = func(self_val, store_vec.as_slice());
            }
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
            cell_bounds: Bounds::empty(),
            cell_boundary_precision: 1e-10,
            rotation_in_parent_rad: 0.0,
            position_in_parent: Vector2::zeros(),
        }
    }
}

impl Bounds {
    /// Creates a new empty (zero sized) bound
    pub fn empty() -> Self {
        Self {
            x: (0, 0),
            y: (0, 0),
        }
    }

    /// Returns if the bounds are valid or not, i.e. if the minimum is larger than the maximum.
    pub fn is_valid(&self) -> bool {
        self.x.0 <= self.x.1 && self.y.0 <= self.y.1
    }

    /// Creates a new bound from the given max and min cell indices in the x and y axes.
    ///
    /// Must satisfy:
    ///  - $x_0 <= x_1$
    ///  - $y_0 <= y_1$
    pub fn new(x: (isize, isize), y: (isize, isize)) -> Result<Self, Error> {
        let bounds = Self { x, y };

        if bounds.is_valid() {
            Ok(bounds)
        } else {
            Err(Error::InvalidBounds(bounds))
        }
    }

    /// Creates a new bound from the given opposing corners of the a rectangle.
    ///
    /// If the corners do not satisfy `all(bottom_left <= upper_right)` the bounds will be invalid
    /// and an error is returned.
    pub fn from_corners(
        bottom_left: Point2<isize>,
        upper_right: Point2<isize>,
    ) -> Result<Self, Error> {
        let bounds = Self {
            x: (bottom_left.x, upper_right.x),
            y: (bottom_left.y, upper_right.y),
        };

        if bounds.is_valid() {
            Ok(bounds)
        } else {
            Err(Error::InvalidBounds(bounds))
        }
    }

    /// Creates a new bound from the given opposing corners of the a rectangle, but the corners do
    /// not have to be sorted in bottom_left, upper_right order.
    ///
    /// This function will automatically decide which points are provided such that the bounds will
    /// be valid.
    pub fn from_corners_unsorted(a: Point2<isize>, b: Point2<isize>) -> Self {
        Self {
            x: (a.x.min(b.x), a.x.max(b.x)),
            y: (a.y.min(b.y), a.y.max(b.y)),
        }
    }

    /// Creates a new bound from the given corner positions, which do not have to be in any order.
    ///
    /// The metadata parameter will be used to map from parent frame position into a map frame.
    pub fn from_corner_positions(
        metadata: &CellMapMetadata,
        a: Point2<f64>,
        b: Point2<f64>,
    ) -> Self {
        // Get the map-rel cell of each point
        let cell_a = metadata.get_cell(a);
        let cell_b = metadata.get_cell(b);

        // Build the bounds
        Self::from_corners_unsorted(cell_a, cell_b)
    }

    /// Converts this bounds into a pair of corners, the bottom left and upper right corners
    /// respectively.
    pub fn as_corners(&self) -> (Point2<isize>, Point2<isize>) {
        (
            Point2::new(self.x.0, self.y.0),
            Point2::new(self.x.1, self.y.1),
        )
    }

    /// Checks if the given point is inside the bounds
    pub fn contains(&self, point: Point2<isize>) -> bool {
        self.x.0 <= point.x && point.x < self.x.1 && self.y.0 <= point.y && point.y < self.y.1
    }

    /// Gets the value of the point as an index into an array bounded by this `Bounds`.
    ///
    /// If the point is outside the bounds `None` is returned
    pub fn get_index(&self, point: Point2<isize>) -> Option<Point2<usize>> {
        if self.contains(point) {
            let unchecked = unsafe { self.get_index_unchecked(point) };

            // Have already checked that the point is inside the bounds, no need to check again
            Some(Point2::new(unchecked.x as usize, unchecked.y as usize))
        } else {
            None
        }
    }

    /// Gets the value of the point as an index into an array bounded by this `Bounds`.
    ///
    /// # Safety
    ///
    /// This function will not panic if `point` is outside the map, but use of the result to
    /// index into the map is not guaranteed to be safe. It is possible for this function to return
    /// a negative index value, which would indicate that the cell is outside the map.
    pub unsafe fn get_index_unchecked(&self, point: Point2<isize>) -> Point2<isize> {
        Point2::new(point.x - self.x.0, point.y - self.y.0)
    }

    /// Gets the shape of this rectangle in a format that `ndarray` will accept.
    ///
    /// NOTE: shape order is (y, x), not (x, y).
    pub fn get_shape(&self) -> (usize, usize) {
        (
            (self.y.1 - self.y.0) as usize,
            (self.x.1 - self.x.0) as usize,
        )
    }

    /// Gets the number of cells as a vector.
    pub fn get_num_cells(&self) -> Vector2<usize> {
        let shape = self.get_shape();
        Vector2::new(shape.1, shape.0)
    }

    /// Gets the intersection of self with other, returning `None` if the two do not intersect.
    pub fn intersect(&self, other: &Bounds) -> Option<Bounds> {
        Bounds::new(
            (self.x.0.max(other.x.0), self.x.1.min(other.x.1)),
            (self.y.0.max(other.y.0), self.y.1.min(other.y.1)),
        )
        .ok()
    }

    /// Get the union of `self` with `other`, effectively this is the axis aligned bounding box of
    /// `self` and `other`.
    ///
    /// If both bounds are empty this bound will be empty.
    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds::new(
            (self.x.0.min(other.x.0), self.x.1.max(other.x.1)),
            (self.y.0.min(other.y.0), self.y.1.max(other.y.1)),
        )
        .unwrap_or_default()
    }

    /// Gets the slice of other within self, cropping other so it fits within self.
    ///
    /// Note that slices are a pair of (min, max) half-open bounds that describe the slice into an
    /// array, i.e. they are indices.
    pub fn get_slice_of_other(&self, other: &Bounds) -> Option<Vector2<(usize, usize)>> {
        // First get intersection of the two bounds in the origin frame
        let intersect = self.intersect(other)?;

        // Rebase the intersection to be a slice relative to the start of self, i.e. subtract the
        // min bound on each axis from both min and max of the intersection
        Some(Vector2::new(
            (
                (intersect.x.0 - self.x.0) as usize,
                (intersect.x.1 - self.x.0) as usize,
            ),
            (
                (intersect.y.0 - self.y.0) as usize,
                (intersect.y.1 - self.y.0) as usize,
            ),
        ))
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::empty()
    }
}
