//! Provides the [`Layered`] iterator, which selects a single layer from the wrapped iterator.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use crate::{iterators::CellMapIter, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides an iterator wrapper which only produces cells from a subset of layers in the entire
/// map.
pub struct Layered<L, T, I>
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

impl<L, T, I> Iterator for Layered<L, T, I>
where
    L: Layer,
    I: CellMapIter<L, T>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<L, T, I> CellMapIter<L, T> for Layered<L, T, I>
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

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use nalgebra::Vector2;

    use crate::{CellMap, CellMapParams, Layer};

    #[derive(Clone, Copy)]
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
    fn cell() {
        // Create dummy map
        let map = CellMap::<MyLayers, f64>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(5, 5),
                centre: Vector2::new(0.0, 0.0),
            },
            1.0,
        );

        // Create an iterator over only one layer and check we have all the cells we expect
        assert_eq!(
            map.iter().layer(MyLayers::Layer0).count(),
            map.params.num_cells.x * map.params.num_cells.y,
        );
        assert_eq!(
            map.iter().layer(MyLayers::Layer1).count(),
            map.params.num_cells.x * map.params.num_cells.y,
        );
        assert_eq!(
            map.iter().layer(MyLayers::Layer2).count(),
            map.params.num_cells.x * map.params.num_cells.y,
        );

        // Create an iter over many layers and check the number of cells is right
        assert_eq!(
            map.iter()
                .layers(&[MyLayers::Layer0, MyLayers::Layer1])
                .count(),
            map.params.num_cells.x * map.params.num_cells.y * 2,
        );
        assert_eq!(
            map.iter()
                .layers(&[MyLayers::Layer0, MyLayers::Layer2])
                .count(),
            map.params.num_cells.x * map.params.num_cells.y * 2,
        );
        assert_eq!(
            map.iter()
                .layers(&[MyLayers::Layer0, MyLayers::Layer1, MyLayers::Layer2])
                .count(),
            map.params.num_cells.x * map.params.num_cells.y * 3,
        );
    }
}
