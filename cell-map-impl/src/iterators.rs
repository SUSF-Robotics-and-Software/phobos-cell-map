//! Provides iterators over [`CellMap`]s
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// MODULES
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::iter::Iterator;

use crate::{CellMap, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides an owned iterator over each cell in each layer of a [`CellMap`]
///
/// The iterator produces items in layer-y-x order, i.e. it will produce the entire row of the
/// first layer, then the second row of that layer, and so on until all rows have been produced, at
/// which point it will move to the next layer.
///
/// To iterate over a single layer use the [`LayerIter`] instead.
#[derive(Clone)]
pub struct CellIter<'c, L: Layer, T: Clone> {
    pub(crate) index: (usize, usize, usize),

    pub(crate) map: &'c CellMap<L, T>,
}

/// Provides a mutable iterator over each cell in each layer of a [`CellMap`]
///
/// The iterator produces items in layer-y-x order, i.e. it will produce the entire row of the
/// first layer, then the second row of that layer, and so on until all rows have been produced, at
/// which point it will move to the next layer.
///
/// To iterate over a single layer use the [`LayerIter`] instead.
pub struct CellIterMut<'c, L: Layer, T: 'c> {
    pub(crate) index: (usize, usize, usize),

    pub(crate) map: &'c mut CellMap<L, T>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'c, L: Layer, T: Clone> Iterator for CellIter<'c, L, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (l, y, x) = self.index;

        // Check if we should still be procuding items
        let data = if l < L::NUM_LAYERS
            && y < self.map.params.num_cells.y
            && x < self.map.params.num_cells.x
        {
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

        // If y is now greater than the max y increment layer and set y to 0
        if self.index.1 >= self.map.params.num_cells.y {
            self.index.1 = 0;
            self.index.0 += 1;
        }

        // Return the data
        data
    }
}

impl<'c, L: Layer, T: Clone> Iterator for CellIterMut<'c, L, T> {
    type Item = &'c mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let (l, y, x) = self.index;

        // Check if we should still be procuding items
        let data = if l < L::NUM_LAYERS
            && y < self.map.params.num_cells.y
            && x < self.map.params.num_cells.x
        {
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

        // If y is now greater than the max y increment layer and set y to 0
        if self.index.1 >= self.map.params.num_cells.y {
            self.index.1 = 0;
            self.index.0 += 1;
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

        // Create an iterator
        let map_iter = map.iter();

        // Check there are the right number of items in the inter
        assert_eq!(
            map_iter.clone().count(),
            map.num_cells().x * map.num_cells().y * MyLayers::NUM_LAYERS
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
            1.0,
        );

        // Check we have the right number of items from an iterator
        assert_eq!(
            map.iter_mut().count(),
            map.num_cells().x * map.num_cells().y * MyLayers::NUM_LAYERS
        );

        // Check all elements are 1
        assert!(map.iter().all(|v| v == 1.0));

        // Change all elements to 2
        map.iter_mut().for_each(|v| *v = 2.0);

        // Check all elements are 2
        assert!(map.iter().all(|v| v == 2.0));
    }
}
