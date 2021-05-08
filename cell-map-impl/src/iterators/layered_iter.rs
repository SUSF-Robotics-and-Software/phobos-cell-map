//! Provides the [`LayerdIter`] and [`LayerdIterMut`] iterators.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use crate::{CellMap, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides an owned iterator over each cell in a single layer of a [`CellMap`]
///
/// The iterator produces items in y-x order, i.e. it will produce the entire row of the, then the
/// second row, and so on until all rows have been produced.
///
/// To iterate over the cells in all layers use [`CellIter`] instead.
#[derive(Clone)]
pub struct LayeredIter<'c, L: Layer, T> {
    pub(crate) index: (usize, usize, usize),
    pub(crate) map: &'c CellMap<L, T>,
}

/// Provides a mutable iterator over each cell in a single layer of a [`CellMap`]
///
/// The iterator produces items in y-x order, i.e. it will produce the entire row of the, then the
/// second row, and so on until all rows have been produced.
///
/// To iterate over the cells in all layers use [`CellIterMut`] instead.
pub struct LayeredIterMut<'c, L: Layer, T> {
    pub(crate) index: (usize, usize, usize),
    pub(crate) map: &'c mut CellMap<L, T>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'c, L: Layer, T: Clone> Iterator for LayeredIter<'c, L, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (l, y, x) = self.index;

        // Check if we should still be procuding items
        let data = if y < self.map.params.num_cells.y && x < self.map.params.num_cells.x {
            Some(self.map.data[l][(y, x)].clone())
        } else {
            return None;
        };

        // Increment x
        self.index.2 += 1;

        // If x is now greater than the max x increment y and set x to 0
        if self.index.2 >= self.map.params.num_cells.x {
            self.index.2 = 0;
            self.index.1 += 1;
        }

        // Return the data
        data
    }
}

impl<'c, L: Layer, T: Clone> Iterator for LayeredIterMut<'c, L, T> {
    type Item = &'c mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let (l, y, x) = self.index;

        // Check if we should still be procuding items
        let data = if y < self.map.params.num_cells.y && x < self.map.params.num_cells.x {
            // USE OF UNSAFE:
            //
            // The compiler has no knowledge that we aren't handing out mutable pointers to the
            // same element, so we need to use unsafe and some pointer magic to get around this.
            // This operation is actually safe as the increments below ensure we won't give a
            // mutable reference to the same element twice.
            //
            // See
            // https://stackoverflow.com/questions/63437935/in-rust-how-do-i-create-a-mutable-iterator
            // for some more info.
            unsafe {
                let layer_ptr = self.map.data.as_mut_ptr().add(l);
                Some((&mut *layer_ptr).uget_mut((y, x)))
            }
        } else {
            return None;
        };

        // Increment x
        self.index.2 += 1;

        // If x is now greater than the max x increment y and set x to 0
        if self.index.2 >= self.map.params.num_cells.x {
            self.index.2 = 0;
            self.index.1 += 1;
        }

        // Return the data
        data
    }
}

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use crate::{CellMap, CellMapParams, Layer};

    #[derive(Clone)]
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
        fn index(&self) -> usize {
            match self {
                MyLayers::Layer0 => 0,
                MyLayers::Layer1 => 1,
                MyLayers::Layer2 => 2,
            }
        }
    }

    #[test]
    fn iter() {
        // Crate dummy cell map
        let map = CellMap::<MyLayers, f64>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(5, 5),
                centre: Vector2::new(0.0, 0.0),
            },
            1.0,
        );

        // Create an iterator on the first layer
        let map_iter = map.layerd_iter(MyLayers::Layer0);

        // Check there are the right number of items in the inter
        assert_eq!(
            map_iter.clone().count(),
            map.num_cells().x * map.num_cells().y
        );

        // Check that all cells are 1
        assert!(map_iter.clone().all(|v| v == 1.0));
    }

    #[test]
    fn iter_mut() {
        // Crate dummy cell map
        let mut map = CellMap::<MyLayers, f64>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(5, 5),
                centre: Vector2::new(0.0, 0.0),
            },
            0.0,
        );

        // Make all elements in the first layer 1, the second 2, etc.
        map.layerd_iter_mut(MyLayers::Layer0).for_each(|v| *v = 1.0);
        map.layerd_iter_mut(MyLayers::Layer1).for_each(|v| *v = 2.0);
        map.layerd_iter_mut(MyLayers::Layer2).for_each(|v| *v = 3.0);

        // Check all the elements are correct
        assert!(map.layerd_iter(MyLayers::Layer0).all(|v| v == 1.0));
        assert!(map.layerd_iter(MyLayers::Layer1).all(|v| v == 2.0));
        assert!(map.layerd_iter(MyLayers::Layer2).all(|v| v == 3.0));
    }
}
