//! Provides useful extension traits over types used by [`CellMap`]
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Affine2, Point2, Vector2};

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

/// Provides extension traits to [`ndarray::Affine2<T>`]
pub(crate) trait Affine2Ext {
    fn position(&self, index: Point2<usize>) -> Point2<f64>;
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

impl Affine2Ext for Affine2<f64> {
    fn position(&self, index: Point2<usize>) -> Point2<f64> {
        // Get the centre of the cell, which is + 0.5 cells in the x and y direction.
        let index_centre = index.cast() + Vector2::new(0.5, 0.5);
        self.transform_point(&index_centre)
    }
}

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Point2Ext;
    use nalgebra::{Point2, Vector2};

    #[test]
    fn bounds() {
        let bounds = Vector2::new((1, 8), (1, 8));

        assert!(Point2::new(1, 1).in_bounds(&bounds));
        assert!(Point2::new(1, 7).in_bounds(&bounds));
        assert!(Point2::new(7, 1).in_bounds(&bounds));
        assert!(Point2::new(7, 7).in_bounds(&bounds));
        assert!(!Point2::new(0, 0).in_bounds(&bounds));
        assert!(!Point2::new(0, 8).in_bounds(&bounds));
        assert!(!Point2::new(8, 0).in_bounds(&bounds));
        assert!(!Point2::new(8, 8).in_bounds(&bounds));
    }
}
