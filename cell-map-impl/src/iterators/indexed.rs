//! Provides an interator which produces indexes along with the values of an iterator over
//! [`CellMap`].

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use nalgebra::Vector2;

use crate::{iterators::CellMapIter, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Modified the wrapped interator to return the index of the cell as well as the cell itself.
pub struct Indexed<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T>,
{
    pub(crate) iter: I,

    pub(crate) _phantom: PhantomData<(L, T)>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L, T, I> Iterator for Indexed<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T>,
{
    type Item = ((L, Vector2<usize>), I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        // Gotta access the index first, as next should increment after yeilding
        let index = (
            self.iter.get_layer(),
            Vector2::new(self.iter.get_x(), self.iter.get_y()),
        );

        if let Some(next) = self.iter.next() {
            Some((index, next))
        } else {
            None
        }
    }
}
