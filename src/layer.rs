//! Provies definition of the [`Layer`] trait.

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
/// #[derive(Layer, Clone)]
/// enum MyLayer {
///     Height,
///     Gradient
/// }
/// ```
///
/// [`CellMap`]: crate::CellMap
pub trait Layer: Clone {
    /// Contains the total number of layers possible with this [`Layer`]
    const NUM_LAYERS: usize;

    /// Contains the first layer variant
    const FIRST: Self;

    /// Maps each variant of the enum to a unique layer index, which can be used to get that layer
    /// from the map.
    fn to_index(&self) -> usize;

    /// Maps each layer index into a variant of the layer.
    ///
    /// # Safety
    ///
    /// If the provided index doesn't match a layer this function will panic.
    fn from_index(index: usize) -> Self;

    /// Returns a vector of all layers in index order.
    fn all() -> Vec<Self>;
}
