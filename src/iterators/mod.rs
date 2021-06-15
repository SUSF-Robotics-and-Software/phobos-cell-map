//! Provides iterators over [`CellMap`]s.
//!
//! Iterators are constructed using the `iter` family of functions on [`CellMap`], such as
//! [`CellMap::iter()`] and [`CellMap::window_iter_mut()`]. Which function you use determines
//! which [`Slicer`] the iterator uses, in other words what order and shape the iterated items will
//! be produced in.
//!
//! Once constructed both [`CellMapIter`] and [`CellMapIterMut`] provide functions to modify the
//! which layers are produced, and whether or not the items also produce their indexes. These can
//! be used like iterator combinators.
//!
//! # Examples
//!
//! Iterate over a 3x3 window of items in the `Height` layer, while also returning the indices
//! of the central cell of the window:
//!
//! ```
//! # use cell_map::{CellMap, CellMapParams, Layer};
//! # use nalgebra::Vector2;
//! # #[derive(Layer, Clone, Debug)]
//! # enum MyLayer {
//! #     Height,
//! #     Gradient,
//! #     Roughness
//! # }
//! #
//! # // Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0), with
//! # // all elements initialised to 1.0.
//! # let mut map = CellMap::<MyLayer, f64>::new_from_elem(
//! #     CellMapParams {
//! #         cell_size: Vector2::new(1.0, 1.0),
//! #         num_cells: Vector2::new(5, 5),
//! #         centre: Vector2::new(0.0, 0.0),
//! #     },
//! #     1.0,
//! # );
//! for ((layer, index), height) in map.window_iter(Vector2::new(1, 1)).unwrap().indexed() {
//!     println!("[{:?}, {}, {}] = {}", layer, index.x, index.y, height);
//! }
//! ```
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// MODULES
// ------------------------------------------------------------------------------------------------

pub mod indexed;
pub mod layerers;
pub mod slicers;
#[cfg(test)]
mod tests;

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use layerers::*;
use nalgebra::Vector2;
use slicers::*;

use crate::{CellMap, CellMapError, Layer};

use self::indexed::Indexed;

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// A non-mutable iterator over a [`CellMap`], see [`Slicer`] and [`layerers`] for more information.
pub struct CellMapIter<'m, L, T, R, S>
where
    L: Layer,
    R: Layerer<L>,
    S: Slicer<'m, L, T>,
{
    map: &'m CellMap<L, T>,
    layerer: R,
    slicer: S,
}

/// A mutable iterator over a [`CellMap`], see [`Slicer`] and [`layerers`] for more information.
pub struct CellMapIterMut<'m, L, T, R, S>
where
    L: Layer,
    R: Layerer<L>,
    S: Slicer<'m, L, T>,
{
    map: &'m mut CellMap<L, T>,
    layerer: R,
    slicer: S,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'m, L, T, R, S> CellMapIter<'m, L, T, R, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
    R: Layerer<L>,
{
    pub(crate) fn new_cells(map: &'m CellMap<L, T>) -> CellMapIter<'m, L, T, Many<L>, Cells> {
        CellMapIter {
            map,
            layerer: Many {
                layers: L::all().into(),
            },
            slicer: Cells::from_map(map),
        }
    }

    pub(crate) fn new_windows(
        map: &'m CellMap<L, T>,
        semi_width: Vector2<usize>,
    ) -> Result<CellMapIter<'m, L, T, Many<L>, Windows>, CellMapError> {
        Ok(CellMapIter {
            map,
            layerer: Many {
                layers: L::all().into(),
            },
            slicer: Windows::from_map(map, semi_width)?,
        })
    }

    /// Converts this iterator to use a [`Single`] layerer, produing data from only one layer.
    pub fn layer(self, layer: L) -> CellMapIter<'m, L, T, Single<L>, S> {
        CellMapIter {
            map: self.map,
            layerer: Single { layer },
            slicer: self.slicer,
        }
    }

    /// Converts this iterator to use a [`Many`] layerer, produing data from many layers.
    pub fn layers(self, layers: &[L]) -> CellMapIter<'m, L, T, Many<L>, S> {
        CellMapIter {
            map: self.map,
            layerer: Many {
                layers: layers.to_vec().into(),
            },
            slicer: self.slicer,
        }
    }

    /// Converts this iterator to also produce the index of the iterated item as well as its value.
    pub fn indexed(self) -> CellMapIter<'m, L, T, R, Indexed<'m, L, T, S>> {
        let current_layer = self.layerer.current().unwrap();
        CellMapIter {
            map: self.map,
            layerer: self.layerer,
            slicer: Indexed::new(self.slicer, current_layer),
        }
    }
}

impl<'m, L, T, R, S> CellMapIterMut<'m, L, T, R, S>
where
    L: Layer,
    R: Layerer<L>,
    S: Slicer<'m, L, T>,
{
    pub(crate) fn new_cells(
        map: &'m mut CellMap<L, T>,
    ) -> CellMapIterMut<'m, L, T, Many<L>, Cells> {
        let slicer = Cells::from_map(map);

        CellMapIterMut {
            map,
            layerer: Many {
                layers: L::all().into(),
            },
            slicer,
        }
    }

    pub(crate) fn new_windows(
        map: &'m mut CellMap<L, T>,
        semi_width: Vector2<usize>,
    ) -> Result<CellMapIterMut<'m, L, T, Many<L>, Windows>, CellMapError> {
        let slicer = Windows::from_map(map, semi_width)?;

        Ok(CellMapIterMut {
            map,
            layerer: Many {
                layers: L::all().into(),
            },
            slicer,
        })
    }

    /// Converts this iterator to use a [`Single`] layerer, produing data from only one layer.
    pub fn layer(self, layer: L) -> CellMapIterMut<'m, L, T, Single<L>, S> {
        CellMapIterMut {
            map: self.map,
            layerer: Single { layer },
            slicer: self.slicer,
        }
    }

    /// Converts this iterator to use a [`Many`] layerer, produing data from many layers.
    pub fn layers(self, layers: &[L]) -> CellMapIterMut<'m, L, T, Many<L>, S> {
        CellMapIterMut {
            map: self.map,
            layerer: Many {
                layers: layers.to_vec().into(),
            },
            slicer: self.slicer,
        }
    }

    /// Converts this iterator to use a [`Map`] layerer, which maps data from one layer to another.
    pub fn map_layers(self, from: L, to: L) -> CellMapIterMut<'m, L, T, Map<L>, S> {
        CellMapIterMut {
            map: self.map,
            layerer: Map { from, to },
            slicer: self.slicer,
        }
    }

    /// Converts this iterator to also produce the index of the iterated item as well as its value.
    pub fn indexed(self) -> CellMapIterMut<'m, L, T, R, Indexed<'m, L, T, S>> {
        let current_layer = self.layerer.current().unwrap();
        CellMapIterMut {
            map: self.map,
            layerer: self.layerer,
            slicer: Indexed::new(self.slicer, current_layer),
        }
    }
}

// ------------------------------------------------------------------------------------------------
// ITERATORS
// ------------------------------------------------------------------------------------------------

impl<'m, L, T, S> Iterator for CellMapIter<'m, L, T, Single<L>, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
{
    type Item = S::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self
            .slicer
            .slice(&self.map.data[self.layerer.layer.to_index()]);

        self.slicer.advance();

        item
    }
}

impl<'m, L, T, S> Iterator for CellMapIterMut<'m, L, T, Single<L>, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
{
    type Item = S::OutputMut;

    fn next(&mut self) -> Option<Self::Item> {
        // Note: use of unsafe
        //
        // We must guarantee that we don't hand out multiple mutable references to the data stored
        // in the map, which we can do since each call to this function will drop the previously
        // returned reference first.
        let item = unsafe {
            let layer_ptr = self
                .map
                .data
                .as_mut_ptr()
                .add(self.layerer.layer.to_index());
            self.slicer.slice_mut(&mut *layer_ptr)
        };

        self.slicer.advance();

        item
    }
}

impl<'m, L, T, S> Iterator for CellMapIter<'m, L, T, Many<L>, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
{
    type Item = S::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self
            .slicer
            .slice(&self.map.data[self.layerer.layers.front()?.to_index()]);

        self.slicer.advance();

        if self.slicer.index().is_none() {
            self.layerer.layers.pop_front();
            self.slicer.reset(self.layerer.current());
        }

        item
    }
}

impl<'m, L, T, S> Iterator for CellMapIterMut<'m, L, T, Many<L>, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
{
    type Item = S::OutputMut;

    fn next(&mut self) -> Option<Self::Item> {
        // Note: use of unsafe
        //
        // We must guarantee that we don't hand out multiple mutable references to the data stored
        // in the map, which we can do since each call to this function will drop the previously
        // returned reference first.
        let item = unsafe {
            let layer_ptr = self
                .map
                .data
                .as_mut_ptr()
                .add(self.layerer.layers.front()?.to_index());
            self.slicer.slice_mut(&mut *layer_ptr)
        };

        self.slicer.advance();

        if self.slicer.index().is_none() {
            self.layerer.layers.pop_front();
            self.slicer.reset(self.layerer.current());
        }

        item
    }
}

impl<'m, L, T, S> Iterator for CellMapIterMut<'m, L, T, Map<L>, S>
where
    L: Layer,
    S: Slicer<'m, L, T>,
{
    type Item = (S::Output, S::OutputMut);

    fn next(&mut self) -> Option<Self::Item> {
        // Note: use of unsafe
        //
        // We must guarantee that we don't hand out multiple mutable references to the data stored
        // in the map, which we can do since each call to this function will drop the previously
        // returned reference first.
        let (from, to) = unsafe {
            let from_ptr = self.map.data.as_ptr().add(self.layerer.from.to_index());
            let from = self.slicer.slice(&*from_ptr);
            let to_ptr = self.map.data.as_mut_ptr().add(self.layerer.to.to_index());
            let to = self.slicer.slice_mut(&mut *to_ptr);

            (from, to)
        };

        self.slicer.advance();

        match (from, to) {
            (Some(f), Some(t)) => Some((f, t)),
            (_, _) => None,
        }
    }
}
