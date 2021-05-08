//! Provides the [`Layer`] trait used to describe the layers within a [`CellMap`].
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Trait which defines the type of the layer index.
///
/// Layer indices should be `enum`s which implement the [`Layer`] trait. By limiting indices to
/// `enums` we can guarentee that all possible values of layer exist, and therefore do not have to
/// provide run time checking that the layer exists within the map.
///
/// This is enforced by the use of the `#[derive(Layer)]` macro, which can only be implemented on
/// `enums`.
///
/// # Safety
///
/// Do not manually implement this trait for non-enum types, as [`CellMap`] will be unable to
/// guarentee that the layer you're attempting to access will be present in the map.
///
/// # Example
/// ```
/// use cell_map::Layer;
///
/// #[derive(Layer)]
/// enum MyLayer {
///     Height,
///     Gradient
/// }
/// ```
///
/// [`CellMap`]: crate::CellMap
pub trait Layer {
    /// Contains the total number of layers possible with this [`Layer`]
    const NUM_LAYERS: usize;

    /// Maps each variant of the enum to a unique layer index, which can be used to get that layer
    /// from the map.
    fn index(&self) -> usize;
}
