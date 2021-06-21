//! Provides the [`Positioned`] wrapper type which modifies a [`Slicer`] to produce the current
//! position as well as the value.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::marker::PhantomData;

use nalgebra::{Affine2, Point2};

use crate::{extensions::Affine2Ext, iterators::Slicer, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// A [`Slicer`] which wrapps another [`Slicer`] and modifies it to produce the position of the item
/// as well as the item itself.
#[derive(Debug, Clone, Copy)]
pub struct Positioned<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    slicer: S,
    layer: L,
    to_parent: Affine2<f64>,
    _phantom: PhantomData<(L, &'a T)>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<'a, L, T, S> Positioned<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    pub(crate) fn new(slicer: S, layer: L, to_parent: Affine2<f64>) -> Self {
        Self {
            slicer,
            layer,
            to_parent,
            _phantom: PhantomData,
        }
    }
}

impl<'a, L, T, S> Slicer<'a, L, T> for Positioned<'a, L, T, S>
where
    L: Layer,
    S: Slicer<'a, L, T>,
{
    type Output = ((L, Point2<f64>), S::Output);

    type OutputMut = ((L, Point2<f64>), S::OutputMut);

    fn slice(&self, data: &'a ndarray::Array2<T>) -> Option<Self::Output> {
        let item = self.slicer.slice(data)?;
        let index = self.slicer.index()?;

        Some(((self.layer.clone(), self.to_parent.position(index)), item))
    }

    fn slice_mut(&self, data: &'a mut ndarray::Array2<T>) -> Option<Self::OutputMut> {
        let item = self.slicer.slice_mut(data)?;
        let index = self.slicer.index()?;

        Some(((self.layer.clone(), self.to_parent.position(index)), item))
    }

    fn advance(&mut self) {
        self.slicer.advance()
    }

    fn index(&self) -> Option<nalgebra::Point2<usize>> {
        self.slicer.index()
    }

    fn reset(&mut self, layer: Option<L>) {
        if let Some(ref l) = layer {
            self.layer = l.clone()
        }

        self.slicer.reset(layer)
    }
}
