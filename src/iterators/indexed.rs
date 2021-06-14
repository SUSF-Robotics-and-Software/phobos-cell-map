//! Provides the [`Indexed`] wrapper type which modifies a [`Slicer`] to produce the current index
//! as well as the value.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use nalgebra::Vector2;

use crate::{iterators::Slicer, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// A [`Slicer`] which wrapps another [`Slicer`] and modifies it to produce the index of the item
/// as well as the item itself.
pub struct Indexed<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    pub(crate) slicer: S,
    pub(crate) layer: L,
    _phantom: PhantomData<(L, &'a T)>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'a, L, T, S> Indexed<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    pub(crate) fn new(slicer: S, layer: L) -> Self {
        Self {
            slicer,
            layer,
            _phantom: PhantomData,
        }
    }
}

impl<'a, L, T, S> Slicer<'a, L, T> for Indexed<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    type Output = ((L, Vector2<usize>), S::Output);

    type OutputMut = ((L, Vector2<usize>), S::OutputMut);

    fn slice(&self, data: &'a ndarray::Array2<T>) -> Option<Self::Output> {
        let item = self.slicer.slice(data)?;

        Some(((self.layer.clone(), self.slicer.index().unwrap()), item))
    }

    fn slice_mut(&self, data: &'a mut ndarray::Array2<T>) -> Option<Self::OutputMut> {
        let item = self.slicer.slice_mut(data)?;

        Some(((self.layer.clone(), self.slicer.index().unwrap()), item))
    }

    fn advance(&mut self) {
        self.slicer.advance()
    }

    fn index(&self) -> Option<nalgebra::Vector2<usize>> {
        self.slicer.index()
    }

    fn reset(&mut self, layer: Option<L>) {
        match layer {
            Some(ref l) => self.layer = l.clone(),
            None => (),
        }

        self.slicer.reset(layer)
    }
}
