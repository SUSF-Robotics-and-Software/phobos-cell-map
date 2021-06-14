//! Provides layerer types which are used in combination with [`Slicer`] types to determine the
//! order and form in which data is produced from the layers within a [`CellMap`]
//!
//! [`CellMap`]: crate::CellMap
//! [`Slicer`]: crate::iterators::slicers::Slicer

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use std::collections::VecDeque;

use crate::Layer;

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// [`Layerer`] controls how items are iterated over a [`CellMap`]s layers.
///
/// [`CellMap`]: crate::CellMap
pub trait Layerer<L>
where
    L: Layer,
{
    /// Returns the current layer.
    fn current(&self) -> Option<L>;
}

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Produces data from a single layer in a [`CellMap`].
///
/// [`CellMap`]: crate::CellMap
pub struct Single<L>
where
    L: Layer,
{
    pub(crate) layer: L,
}

/// Produces data from `N` layers in a [`CellMap`]
///
/// The data is produced in [`Layer::to_index()`] order.
///
/// [`CellMap`]: crate::CellMap

pub struct Many<L>
where
    L: Layer,
{
    pub(crate) layers: VecDeque<L>,
}

/// Produces data from two layers in pairs of `(&from, &mut to)`, allowing you to map data from one
/// layer into another.
pub struct Map<L>
where
    L: Layer,
{
    pub(crate) from: L,
    pub(crate) to: L,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L> Layerer<L> for Single<L>
where
    L: Layer,
{
    fn current(&self) -> Option<L> {
        Some(self.layer.clone())
    }
}

impl<L> Layerer<L> for Many<L>
where
    L: Layer,
{
    fn current(&self) -> Option<L> {
        self.layers.front().map(|l| l.clone())
    }
}

impl<L> Layerer<L> for Map<L>
where
    L: Layer,
{
    fn current(&self) -> Option<L> {
        Some(self.from.clone())
    }
}
