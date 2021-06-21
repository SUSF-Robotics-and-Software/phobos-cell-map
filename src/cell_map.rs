//! # `CellMap` implementation

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    usize,
};

use nalgebra::{Affine2, Isometry2, Matrix3, Point2, Translation2, UnitComplex, Vector2};
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

    /// The transform between the map's frame and the parent frame. This is the transform that will
    /// be applied when going from a cell index to a parent-frame position, i.e.:
    ///
    /// $$ \vec{p_{parent}} = \mathtt{toparent}\ \hat{p_{map}} $$
    to_parent: Affine2<f64>,

    layer_type: PhantomData<L>,
}

/// Contains parameters required to construct a [`CellMap`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CellMapParams {
    /// The size (resolution) of each cell in the map, in both the `x` and `y` directions.
    pub cell_size: Vector2<f64>,

    /// The number of cells in the `x` and `y` directions.
    pub num_cells: Vector2<usize>,

    /// The rotation (unit complex number of magnitude 1) that goes from the map frame to the
    /// parent frame.
    pub to_parent_rotation: UnitComplex<f64>,

    /// The translation that will go from the map origin to the parent frame origin.
    pub to_parent_translation: Translation2<f64>,

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
    /// This value defaults to `1e-10`.
    pub cell_boundary_precision: f64,
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
        self.params.cell_size
    }

    /// Returns the number of cells in each direction of the map.
    pub fn num_cells(&self) -> Vector2<usize> {
        self.params.num_cells
    }

    /// Gets the [`nalgebra::Affine2<f64>`] transformation between the map frame and the parent
    /// frame.
    pub fn to_parent(&self) -> Affine2<f64> {
        self.to_parent
    }

    /// Returns whether or not the given index is inside the map.
    pub fn is_in_map(&self, index: Point2<usize>) -> bool {
        index.x < self.params.num_cells.x && index.y < self.params.num_cells.y
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
        if self.is_in_map(index) {
            Some(self.position_unchecked(index))
        } else {
            None
        }
    }

    /// Returns the position in the parent frame of the centre of the given cell index, without
    /// checking that the `index` is inside the map.
    ///
    /// # Safety
    ///
    /// This method won't panic if `index` is outside the map, but it's result can't be guaranteed
    /// to be a position in the map.
    pub fn position_unchecked(&self, index: Point2<usize>) -> Point2<f64> {
        // Get the centre of the cell, which is + 0.5 cells in the x and y direction.
        let index_centre = index.cast() + Vector2::new(0.5, 0.5);
        self.to_parent.transform_point(&index_centre)
    }

    /// Get the cell index of the given poisition.
    ///
    /// Returns `None` if the given `position` is not inside the map.
    pub fn index(&self, position: Point2<f64>) -> Option<Point2<usize>> {
        let index = unsafe { self.index_unchecked(position) };

        if index.x < 0 || index.y < 0 {
            return None;
        }

        let index = index.map(|v| v as usize);

        if self.is_in_map(index) {
            Some(index)
        } else {
            None
        }
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
        let els: Vec<isize> = self
            .to_parent
            .inverse_transform_point(&position)
            .iter()
            .zip(self.cell_size().iter())
            .map(|(&v, &s)| {
                let v_floor = v as isize;
                let v_next_floor = (s * self.params.cell_boundary_precision + v) as isize;

                if v_floor != v_next_floor {
                    v_next_floor
                } else {
                    v_floor
                }
            })
            .collect();
        Point2::new(els[0], els[1])
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
            to_parent: params.get_affine(),
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
            to_parent: params.get_affine(),
        }
    }
}

impl CellMapParams {
    fn get_affine(&self) -> Affine2<f64> {
        // First build isometry
        let isom = Isometry2::from_parts(self.to_parent_translation, self.to_parent_rotation);

        // Scale transformation matrix, based on cell size.
        let scale = Matrix3::new(
            self.cell_size.x,
            0.0,
            0.0,
            0.0,
            self.cell_size.y,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        // Build the affine by multiplying isom and scale, which will take the translation and
        // rotation of isom and scale it by the cell size. Scale must come first so that the isom,
        // which is in parent coordinates, is not scaled itself.
        Affine2::from_matrix_unchecked(isom.to_matrix() * scale)
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
            cell_size: Vector2::zeros(),
            num_cells: Vector2::zeros(),
            to_parent_rotation: UnitComplex::identity(),
            to_parent_translation: Translation2::identity(),
            cell_boundary_precision: 1e-10,
        }
    }
}
