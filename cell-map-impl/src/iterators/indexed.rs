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
        // Gotta access the index first, as next should increment after yeilding. Must use the
        // get_layer_checked function instead of get layer to avoid panics once the iterator has
        // reached the end of the map.
        let index = (
            self.get_layer_checked()?,
            Vector2::new(self.iter.get_x(), self.iter.get_y()),
        );

        if let Some(next) = self.iter.next() {
            Some((index, next))
        } else {
            return None;
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

    fn get_layer_checked(&self) -> Option<L> {
        self.iter.get_layer_checked()
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

impl<L, T, I> Clone for Indexed<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            _phantom: PhantomData,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use nalgebra::Vector2;
    use ndarray::arr2;

    use crate::{CellMap, CellMapParams, Layer};

    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
    #[allow(dead_code)]
    enum MyLayers {
        Layer0,
        Layer1,
        Layer2,
    }

    // Have to do a manual impl because the derive doesn't like working inside this crate, for some
    // reason
    impl Layer for MyLayers {
        const NUM_LAYERS: usize = 3;
        const FIRST: Self = Self::Layer0;
        fn to_index(&self) -> usize {
            match self {
                Self::Layer0 => 0,
                Self::Layer1 => 1,
                Self::Layer2 => 2,
            }
        }

        fn from_index(index: usize) -> Self {
            match index {
                0 => Self::Layer0,
                1 => Self::Layer1,
                2 => Self::Layer2,
                _ => panic!(
                    "Got a layer index of {} but there are only {} layers",
                    index,
                    Self::NUM_LAYERS
                ),
            }
        }
    }

    #[test]
    fn index_correct() {
        // Create dummy map
        let mut map = CellMap::<MyLayers, usize>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(3, 3),
                centre: Vector2::new(0.0, 0.0),
            },
            0,
        );

        // Set each element in the map to the sum of it's indices
        map.iter_mut().indexed().for_each(|((layer, cell), value)| {
            *value = layer.to_index() + cell.x + cell.y;
        });

        // Check that each cell is now set correctly
        assert_eq!(
            map[MyLayers::Layer0],
            arr2(&[[0, 1, 2], [1, 2, 3], [2, 3, 4]])
        );
        assert_eq!(
            map[MyLayers::Layer1],
            arr2(&[[1, 2, 3], [2, 3, 4], [3, 4, 5]])
        );
        assert_eq!(
            map[MyLayers::Layer2],
            arr2(&[[2, 3, 4], [3, 4, 5], [4, 5, 6]])
        );
    }
}
