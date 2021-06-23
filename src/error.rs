//! Defines the standard error type for errors related to [`CellMap`]s.
//!
//! [`CellMap`]: crate::CellMap

use nalgebra::{Point2, Vector2};

// ------------------------------------------------------------------------------------------------
// ENUMS
// ------------------------------------------------------------------------------------------------

/// Standard error type for errors related to [`CellMap`]s.
///
/// [`CellMap`]: crate::CellMap
#[derive(Clone, Debug, thiserror::Error)]
pub enum CellMapError {
    /// Error returned when trying to construct a [`Windows`] slicer using a `semi_width` which
    /// would create a window larger than the size of the map.
    ///
    /// [`Windows`]: crate::iterators::slicers::Windows
    #[error("Can't create a Windows iterator since the window size ({0}) is larger than the map size ({1})")]
    WindowLargerThanMap(Vector2<usize>, Vector2<usize>),

    /// The given parent-frame position (name, first element) is outside the map.
    #[error("Parent-frame position {0} ({1}) is outside the map")]
    PositionOutsideMap(String, Point2<f64>),
}
