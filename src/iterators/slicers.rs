//! Provides the [`Slicer`] trait and slicers types that determine the order and pattern in which
//! data is produced by an interator over a [`CellMap`].
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::Vector2;
use ndarray::{s, Array2, ArrayView2, ArrayViewMut2};

use crate::{extensions::Vector2Ext, CellMap, CellMapError, Layer};

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Trait which allows a [`CellMapIter`] or [`CellMapIterMut`] struct to determine what shape the
/// data in the iteration should be produced in. For example:
///
/// - [`Cells`] produces data in each cell in `(x, y)` order, x increasing most rapidly.
/// - [`Windows`] produces rectangular views in `(x, y`) order, x increasing most rapidly.
///
/// [`Slicer`]s are designed to be used in iterators, so after a `.slice` or `.slice_mut` the user
/// shall call [`Slicer::advance()`] on the type.
///
/// [`CellMapIter`]: crate::iterators::CellMapIter
/// [`CellMapIterMut`]: crate::iterators::CellMapIterMut
pub trait Slicer<'a, L, T>
where
    L: Layer,
{
    /// The non-mutable output type for the data of this [`Slicer`].
    type Output;

    /// The mutable output type for the data of this [`Slicer`].
    type OutputMut;

    /// Perform the slice on the given `data` layer, or `None` if the slicer has reached the end of
    /// its data.
    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output>;

    /// Perform a mutable slice on the given `data` layer, or `None` if the slicer has reached the
    /// end of its data.
    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut>;

    /// Advance the [`Slicer`] to the next index.
    fn advance(&mut self);

    /// Return the current index of the [`Slicer`], or `None` if the slicer has reached the end of
    /// its data.
    fn index(&self) -> Option<Vector2<usize>>;

    /// Resets the index of this [`Slicer`] so that it can be used on the next layer in the
    /// iteration. The `layer` input is used for slicers which need to monitor which layer they are
    /// on.
    fn reset(&mut self, layer: Option<L>);
}

pub(crate) type RectBounds = Vector2<(usize, usize)>;

/// A [`Slicer`] which produces cells in `(x, y)` order inside a layer, with `x` increasing most
/// rapidly.
pub struct Cells {
    bounds: RectBounds,
    index: Vector2<usize>,
}

/// A [`Slicer`] which produces rectangular views into a layer in `(x, y)` order, increasing `x`
/// most rapidly. A boundary of the `semi_width` of the window around the outside edge of the map
/// is used to prevent indexing outside the map.
pub struct Windows {
    bounds: RectBounds,
    index: Vector2<usize>,
    semi_width: Vector2<usize>,
}

impl Cells {
    pub(crate) fn from_map<L: Layer, T>(map: &CellMap<L, T>) -> Self {
        let cells = map.num_cells();
        Self {
            bounds: Vector2::new((0, cells.x), (0, cells.y)),
            index: Vector2::new(0, 0),
        }
    }
}

impl<'a, L, T> Slicer<'a, L, T> for Cells
where
    L: Layer,
    T: 'a,
{
    type Output = &'a T;
    type OutputMut = &'a mut T;

    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output> {
        data.get(self.index.as_array2_index())
    }

    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut> {
        data.get_mut(self.index.as_array2_index())
    }

    fn advance(&mut self) {
        self.index.x += 1;

        if !self.index.in_bounds(&self.bounds) {
            self.index.y += 1;
            self.index.x = self.bounds.x.0;
        }
    }

    fn index(&self) -> Option<Vector2<usize>> {
        if self.index.in_bounds(&self.bounds) {
            Some(self.index)
        } else {
            None
        }
    }

    fn reset(&mut self, _layer: Option<L>) {
        self.index = Vector2::new(self.bounds.x.0, self.bounds.y.0);
    }
}

impl Windows {
    pub(crate) fn from_map<L: Layer, T>(
        map: &CellMap<L, T>,
        semi_width: Vector2<usize>,
    ) -> Result<Self, CellMapError> {
        let cells = map.num_cells();

        if semi_width.x * 2 + 1 > cells.x || semi_width.y * 2 + 1 > cells.y {
            Err(CellMapError::WindowLargerThanMap(
                semi_width * 2 + Vector2::new(1, 1),
                cells,
            ))
        } else {
            let bounds = Vector2::new(
                (semi_width.x, cells.x - semi_width.x),
                (semi_width.y, cells.y - semi_width.y),
            );

            Ok(Self {
                bounds,
                index: Vector2::new(bounds.x.0, bounds.y.0),
                semi_width,
            })
        }
    }
}

impl<'a, L, T> Slicer<'a, L, T> for Windows
where
    L: Layer,
    T: 'a,
{
    type Output = ArrayView2<'a, T>;
    type OutputMut = ArrayViewMut2<'a, T>;

    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output> {
        if self.index.in_bounds(&self.bounds) {
            let x0 = self.index.x - self.semi_width.x;
            let x1 = self.index.x + self.semi_width.x;
            let y0 = self.index.y - self.semi_width.y;
            let y1 = self.index.y + self.semi_width.y;
            Some(data.slice(s![y0..y1, x0..x1]))
        } else {
            None
        }
    }

    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut> {
        if self.index.in_bounds(&self.bounds) {
            let x0 = self.index.x - self.semi_width.x;
            let x1 = self.index.x + self.semi_width.x;
            let y0 = self.index.y - self.semi_width.y;
            let y1 = self.index.y + self.semi_width.y;
            Some(data.slice_mut(s![y0..y1, x0..x1]))
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.index.x += 1;

        if !self.index.in_bounds(&self.bounds) {
            self.index.y += 1;
            self.index.x = self.bounds.x.0;
        }
    }

    fn index(&self) -> Option<Vector2<usize>> {
        if self.index.in_bounds(&self.bounds) {
            Some(self.index)
        } else {
            None
        }
    }

    fn reset(&mut self, _layer: Option<L>) {
        self.index = Vector2::new(self.bounds.x.0, self.bounds.y.0);
    }
}
