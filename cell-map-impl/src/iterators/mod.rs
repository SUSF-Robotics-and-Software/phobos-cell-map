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

    /// Return the current layer of the iterator.
    ///
    /// # Safety
    ///
    /// This function will panic if the current layer is out of bounds, use `get_layer_checked` to
    /// perform this check without the panic.
    #[doc(hidden)]
    fn get_layer(&self) -> L;

    /// Returns the current layer of the iterator, or `None` if the layer is out of bounds.
    #[doc(hidden)]
    fn get_layer_checked(&self) -> Option<L>;

    /// Return the current x coordinate of the iterator
    #[doc(hidden)]
    fn get_x(&self) -> usize;

    /// Return the current y coordinate of the iterator
    #[doc(hidden)]
    fn get_y(&self) -> usize;
}
