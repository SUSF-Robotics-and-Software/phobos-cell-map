//! Provides an interator which produces indexes along with the values of an iterator over
//! [`CellMap`].

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use nalgebra::Vector2;

use crate::{iterators::CellMapIter, Layer};

use super::Layered;

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

impl<L, T, I> CellMapIter<L, T> for Indexed<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T>,
{
    fn limit_layers(&mut self, layers: &[L]) {
        self.iter.limit_layers(layers)
    }

    fn get_layer(&self) -> L {
        self.iter.get_layer()
    }

    fn get_x(&self) -> usize {
        self.iter.get_x()
    }

    fn get_y(&self) -> usize {
        self.iter.get_y()
    }
}

impl<L, T, I> Indexed<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T>,
{
    /// Modifies this iterator to only produce the cells in the given layer.
    pub fn layer(mut self, layer: L) -> Layered<L, T, Self> {
        self.limit_layers(&[layer]);
        Layered {
            iter: self,
            _phantom: PhantomData,
        }
    }

    /// Modifies this iterator to only produce the cells in the given layers.
    pub fn layers(mut self, layers: &[L]) -> Layered<L, T, Self> {
        self.limit_layers(layers);
        Layered {
            iter: self,
            _phantom: PhantomData,
        }
    }
}
