//! Provides the [`CellMapMetadata`] struct which describes metadata about a [`CellMap`], such as
//! its location and size.
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Affine2, Isometry2, Matrix3, Point2, Vector2};
use serde::{Deserialize, Serialize};

use crate::{iterators::slicers::RectBounds, CellMapParams};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides metadata about a [`CellMap`], such as size and location.
///
/// The data in this struct is constructed from the [`CellMapParams`] provided by the user at
/// construction of the map.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct CellMapMetadata {
    /// The size (resolution) of each cell in the map, in both the `x` and `y` directions.
    pub cell_size: Vector2<f64>,

    /// The number of cells in the `x` and `y` directions.
    pub num_cells: Vector2<usize>,

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

    /// The transform between the map's frame and the parent frame. This is the transform that will
    /// be applied when going from a cell index to a parent-frame position.
    pub to_parent: Affine2<f64>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl CellMapMetadata {
    /// Returns the bounds of the map in map frame coordinates.
    pub fn get_bounds(&self) -> RectBounds {
        Vector2::new((0, self.num_cells.x), (0, self.num_cells.y))
    }

    /// Returns whether or not the given index is inside the map.
    pub fn is_in_map(&self, index: Point2<usize>) -> bool {
        index.x < self.num_cells.x && index.y < self.num_cells.y
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
            .zip(self.cell_size.iter())
            .map(|(&v, &s)| {
                let v_floor = v as isize;
                let v_next_floor = (s * self.cell_boundary_precision + v) as isize;

                if v_floor != v_next_floor {
                    v_next_floor
                } else {
                    v_floor
                }
            })
            .collect();
        Point2::new(els[0], els[1])
    }
}

impl From<CellMapParams> for CellMapMetadata {
    fn from(params: CellMapParams) -> Self {
        // First build isometry to convert from the parent to map
        let isom_from_parent =
            Isometry2::new(params.position_in_parent, params.rotation_in_parent_rad);

        // Scale transformation matrix, based on cell size.
        let scale = Matrix3::new(
            params.cell_size.x,
            0.0,
            0.0,
            0.0,
            params.cell_size.y,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        // Build the affine by multiplying isom and scale, which will take the translation and
        // rotation of isom and scale it by the cell size. Scale must come first so that the isom,
        // which is in parent coordinates, is not scaled itself. Get the inverse of
        // isom_from_parent to get the to_parent
        let to_parent = Affine2::from_matrix_unchecked(isom_from_parent.to_matrix() * scale);

        Self {
            cell_size: params.cell_size,
            num_cells: params.num_cells,
            cell_boundary_precision: params.cell_boundary_precision,
            to_parent,
        }
    }
}
