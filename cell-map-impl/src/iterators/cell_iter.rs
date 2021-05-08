//! Provides the [`CellIter`] and [`CellIterMut`] iterators.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::{iter::Iterator, marker::PhantomData};

use crate::{CellMap, Layer};

use super::{CellMapIter, Indexed, Layered};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides an owned iterator over each cell in each layer of a [`CellMap`]
///
/// The iterator produces items in layer-y-x order, i.e. it will produce the entire row of the
/// first layer, then the second row of that layer, and so on until all rows have been produced, at
/// which point it will move to the next layer.
#[derive(Clone)]
pub struct CellIter<'c, L: Layer, T: Clone> {
    pub(crate) layer_limits: Option<Vec<L>>,
    pub(crate) limits_idx: Option<usize>,

    pub(crate) index: (usize, usize, usize),

    pub(crate) map: &'c CellMap<L, T>,
}

/// Provides a mutable iterator over each cell in each layer of a [`CellMap`]
///
/// The iterator produces items in layer-y-x order, i.e. it will produce the entire row of the
/// first layer, then the second row of that layer, and so on until all rows have been produced, at
/// which point it will move to the next layer.
pub struct CellIterMut<'c, L: Layer, T> {
    pub(crate) layer_limits: Option<Vec<L>>,
    pub(crate) limits_idx: Option<usize>,

    pub(crate) index: (usize, usize, usize),

    pub(crate) map: &'c mut CellMap<L, T>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'c, L: Layer, T: Clone> Iterator for CellIter<'c, L, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let l = self.index.0;
        let y = self.index.1;
        let x = self.index.2;

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

        // Check if we have limits set on our layers
        if let Some(ref limits) = self.layer_limits {
            // If y is now greater than the max y increment the layer and set y to 0
            if self.index.1 >= self.map.params.num_cells.y {
                self.index.1 = 0;

                // Get the index into layer_limits of the next layer, we can unwrap since
                // limits_idx must be set when we set layer_limits.
                let next_layer_lim_idx = self.limits_idx.unwrap() + 1;

                // If the next index is greater than the length of the layer limits we set the
                // current layer to the number of layers, which will cause the subsequent call to
                // next to return None.
                if next_layer_lim_idx >= limits.len() {
                    self.index.0 = L::NUM_LAYERS;
                } else {
                    self.index.0 = limits[next_layer_lim_idx].to_index();
                    *self.limits_idx.as_mut().unwrap() += 1;
                }
            }
        }
        // If not limimted apply same logic as for x, but for y and layers
        else if self.index.1 >= self.map.params.num_cells.y {
            self.index.1 = 0;
            self.index.0 += 1;
        }

        // Return the data
        data
    }
}

impl<L, T> CellMapIter<L, T> for CellIter<'_, L, T>
where
    L: Layer,
    T: Clone,
{
    fn limit_layers(&mut self, layers: &[L]) {
        self.limits_idx = Some(layers[0].to_index());
        self.layer_limits = Some(layers.into());
    }

    fn get_layer(&self) -> L {
        L::from_index(self.index.0)
    }

    fn get_x(&self) -> usize {
        self.index.2.clone()
    }

    fn get_y(&self) -> usize {
        self.index.1.clone()
    }
}

impl<L, T> CellIter<'_, L, T>
where
    L: Layer,
    T: Clone,
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

    /// Modifies this iterator to produce the index as well as the cell.
    pub fn indexed(self) -> Indexed<L, T, Self> {
        Indexed {
            iter: self,
            _phantom: PhantomData,
        }
    }
}

impl<'c, L: Layer, T: Clone> Iterator for CellIterMut<'c, L, T> {
    type Item = &'c mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let l = self.index.0;
        let y = self.index.1;
        let x = self.index.2;

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

        // Check if we have limits set on our layers
        if let Some(ref limits) = self.layer_limits {
            // If y is now greater than the max y increment the layer and set y to 0
            if self.index.1 >= self.map.params.num_cells.y {
                self.index.1 = 0;

                // Get the index into layer_limits of the next layer, we can unwrap since
                // limits_idx must be set when we set layer_limits.
                let next_layer_lim_idx = self.limits_idx.unwrap() + 1;

                // If the next index is greater than the length of the layer limits we set the
                // current layer to the number of layers, which will cause the subsequent call to
                // next to return None.
                if next_layer_lim_idx >= limits.len() {
                    self.index.0 = L::NUM_LAYERS;
                } else {
                    self.index.0 = limits[next_layer_lim_idx].to_index();
                }
            }
        }
        // If not limimted apply same logic as for x, but for y and layers
        else if self.index.1 >= self.map.params.num_cells.y {
            self.index.1 = 0;
            self.index.0 += 1;
        }

        // Return the data
        data
    }
}

impl<L, T> CellMapIter<L, T> for CellIterMut<'_, L, T>
where
    L: Layer,
    T: Clone,
{
    fn limit_layers(&mut self, layers: &[L]) {
        self.limits_idx = Some(layers[0].to_index());
        self.layer_limits = Some(layers.into());
    }

    fn get_layer(&self) -> L {
        L::from_index(self.index.0)
    }

    fn get_x(&self) -> usize {
        self.index.2.clone()
    }

    fn get_y(&self) -> usize {
        self.index.1.clone()
    }
}

impl<L, T> CellIterMut<'_, L, T>
where
    L: Layer,
    T: Clone,
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

    /// Modifies this iterator to produce the index as well as the cell.
    pub fn indexed(self) -> Indexed<L, T, Self> {
        Indexed {
            iter: self,
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
