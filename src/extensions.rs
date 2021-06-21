//! Provides useful extension traits over types used by [`CellMap`]
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Point2, Vector2};

use crate::iterators::slicers::RectBounds;

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Provides a trait to convert a [`Vector2<T>`] to a shape acceptable to `ndarray`.
pub(crate) trait ToShape {
    fn to_shape(&self) -> (usize, usize);
}

/// Provides extension traits to an [`ndarray::Point2`].
pub(crate) trait Point2Ext {
    fn in_bounds(&self, bounds: &RectBounds) -> bool;

    fn as_array2_index(&self) -> [usize; 2];
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl ToShape for Vector2<usize> {
    fn to_shape(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}

impl Point2Ext for Point2<usize> {
    fn in_bounds(&self, bounds: &RectBounds) -> bool {
        self.x >= bounds.x.0 && self.x < bounds.x.1 && self.y >= bounds.y.0 && self.y < bounds.y.1
    }

    fn as_array2_index(&self) -> [usize; 2] {
        [self.y, self.x]
    }
}
