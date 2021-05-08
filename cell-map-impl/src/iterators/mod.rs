//! Provides iterators over [`CellMap`]s
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// MODULES
// ------------------------------------------------------------------------------------------------

mod cell_iter;
mod indexed;
mod layered;

// ------------------------------------------------------------------------------------------------
// EXPORTS
// ------------------------------------------------------------------------------------------------

pub use cell_iter::{CellIter, CellIterMut};
pub use indexed::Indexed;
pub use layered::Layered;

use crate::Layer;

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Trait which all iterators over [`CellMap`] must implement.
pub trait CellMapIter<L, T>: Iterator
where
    L: Layer,
{
    /// Limits the iterator to only producing the given layers.
    ///
    /// The implementor should also ensure that their current layer is set to the first element of
    /// `layers`.
    #[doc(hidden)]
    fn limit_layers(&mut self, layers: &[L]);

    /// Return the current layer of the iterator
    #[doc(hidden)]
    fn get_layer(&self) -> L;

    /// Return the current x coordinate of the iterator
    #[doc(hidden)]
    fn get_x(&self) -> usize;

    /// Return the current y coordinate of the iterator
    #[doc(hidden)]
    fn get_y(&self) -> usize;
}
