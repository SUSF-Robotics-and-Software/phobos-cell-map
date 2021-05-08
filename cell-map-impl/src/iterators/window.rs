//! Provides windowed iterators [`WindowIter`] and [`WindowIterMut`] for [`CellMap`].

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::Vector2;
use ndarray::{s, ArrayView2, ArrayViewMut2};

use crate::{CellMap, Layer};

use super::CellMapIter;

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Provides an iterator over windows of cells in the map.
pub struct WindowIter<'c, L, T>
where
    L: Layer,
{
    pub(crate) layer_limits: Option<Vec<L>>,
    pub(crate) limits_idx: Option<usize>,

    pub(crate) index: (usize, usize, usize),
    pub(crate) semi_window_size: Vector2<usize>,

    pub(crate) map: &'c CellMap<L, T>,
}

/// Provides a mutable iterator over windows of cells in the map.
pub struct WindowIterMut<'c, L, T>
where
    L: Layer,
{
    pub(crate) layer_limits: Option<Vec<L>>,
    pub(crate) limits_idx: Option<usize>,

    pub(crate) index: (usize, usize, usize),
    pub(crate) semi_window_size: Vector2<usize>,

    pub(crate) map: &'c mut CellMap<L, T>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'c, L, T> Iterator for WindowIter<'c, L, T>
where
    L: Layer,
    T: Clone,
{
    type Item = ArrayView2<'c, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let l = self.index.0;
        let y = self.index.1;
        let x = self.index.2;

        // Check if we should still be procuding items
        let data = if l < L::NUM_LAYERS
            && y < self.map.params.num_cells.y - self.semi_window_size.y
            && x < self.map.params.num_cells.x - self.semi_window_size.x
        {
            Some(self.map.data[l].slice(s![
                y - self.semi_window_size.y..y + self.semi_window_size.y,
                x - self.semi_window_size.x..x + self.semi_window_size.x
            ]))
        } else {
            return None;
        };

        // Increment x
        self.index.2 += 1;

        // If x is now greater than the max x increment y and set x to the semi window width
        if self.index.2 >= self.map.params.num_cells.x - self.semi_window_size.x {
            self.index.2 = self.semi_window_size.x;
            self.index.1 += 1;
        }

        // Check if we have limits set on our layers
        if let Some(ref limits) = self.layer_limits {
            // If y is now greater than the max y increment the layer and set y to 0
            if self.index.1 >= self.map.params.num_cells.y - self.semi_window_size.y {
                self.index.1 = self.semi_window_size.y;

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
        else if self.index.1 >= self.map.params.num_cells.y - self.semi_window_size.y {
            self.index.1 = self.semi_window_size.y;
            self.index.0 += 1;
        }

        // Return the data
        data
    }
}

impl<L, T> CellMapIter<L, T> for WindowIter<'_, L, T>
where
    L: Layer,
    T: Clone,
{
    fn limit_layers(&mut self, layers: &[L]) {
        self.limits_idx = Some(0);
        self.index.0 = layers[0].to_index();
        self.layer_limits = Some(layers.into());
    }

    fn get_layer(&self) -> L {
        L::from_index(self.index.0)
    }

    fn get_layer_checked(&self) -> Option<L> {
        if self.index.0 < L::NUM_LAYERS {
            Some(L::from_index(self.index.0))
        } else {
            None
        }
    }

    fn get_x(&self) -> usize {
        self.index.2
    }

    fn get_y(&self) -> usize {
        self.index.1
    }
}

impl<'c, L, T> Iterator for WindowIterMut<'c, L, T>
where
    L: Layer,
{
    type Item = ArrayViewMut2<'c, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let l = self.index.0;
        let y = self.index.1;
        let x = self.index.2;

        // Check if we should still be procuding items
        let data = if l < L::NUM_LAYERS
            && y < self.map.params.num_cells.y - self.semi_window_size.y
            && x < self.map.params.num_cells.x - self.semi_window_size.x
        {
            // USE OF UNSAFE:
            // TODO: Not sure if this reasoning is sound for windowed...
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
                Some((&mut *layer_ptr).slice_mut(s![
                    y - self.semi_window_size.y..y + self.semi_window_size.y,
                    x - self.semi_window_size.x..x + self.semi_window_size.x
                ]))
            }
        } else {
            return None;
        };

        // Increment x
        self.index.2 += 1;

        // If x is now greater than the max x increment y and set x to the semi window width
        if self.index.2 >= self.map.params.num_cells.x - self.semi_window_size.x {
            self.index.2 = self.semi_window_size.x;
            self.index.1 += 1;
        }

        // Check if we have limits set on our layers
        if let Some(ref limits) = self.layer_limits {
            // If y is now greater than the max y increment the layer and set y to 0
            if self.index.1 >= self.map.params.num_cells.y - self.semi_window_size.y {
                self.index.1 = self.semi_window_size.y;

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
        else if self.index.1 >= self.map.params.num_cells.y - self.semi_window_size.y {
            self.index.1 = self.semi_window_size.y;
            self.index.0 += 1;
        }

        // Return the data
        data
    }
}

impl<L, T> CellMapIter<L, T> for WindowIterMut<'_, L, T>
where
    L: Layer,
{
    fn limit_layers(&mut self, layers: &[L]) {
        self.limits_idx = Some(0);
        self.index.0 = layers[0].to_index();
        self.layer_limits = Some(layers.into());
    }

    fn get_layer(&self) -> L {
        L::from_index(self.index.0)
    }

    fn get_layer_checked(&self) -> Option<L> {
        if self.index.0 < L::NUM_LAYERS {
            Some(L::from_index(self.index.0))
        } else {
            None
        }
    }

    fn get_x(&self) -> usize {
        self.index.2
    }

    fn get_y(&self) -> usize {
        self.index.1
    }
}

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;
    use ndarray::s;

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
    fn iter() {
        // Create dummy map
        let map = CellMap::<MyLayers, f64>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(5, 5),
                centre: Vector2::new(0.0, 0.0),
            },
            1.0,
        );

        // If we iterate a window of size 1 over this we should get 9 total windows, since we
        // exclude the outer border from the iteration. Accounting for each layer that's 27 views
        assert_eq!(map.window_iter(Vector2::new(1, 1)).count(), 27);
    }

    #[test]
    fn iter_mut() {
        // Create dummy map
        let mut map = CellMap::<MyLayers, f64>::new_from_elem(
            CellMapParams {
                cell_size: Vector2::new(1.0, 1.0),
                num_cells: Vector2::new(5, 5),
                centre: Vector2::new(0.0, 0.0),
            },
            1.0,
        );

        // If we iterate a window of size 1 over this we should get 9 total windows, since we
        // exclude the outer border from the iteration. Accounting for each layer that's 27 views
        assert_eq!(map.window_iter(Vector2::new(1, 1)).count(), 27);

        // Now set everything in the window to be 2
        map.window_iter_mut(Vector2::new(1, 1)).for_each(|mut v| {
            v[(1, 1)] = 2.0;
        });

        // Check that everything except those on the edge are 2
        for ((_, cell), value) in map.iter().indexed() {
            if cell.x == 0 || cell.x == 4 || cell.y == 0 || cell.y == 4 {
                assert_eq!(value, 1.0)
            } else {
                assert_eq!(value, 2.0)
            }
        }
    }
}
