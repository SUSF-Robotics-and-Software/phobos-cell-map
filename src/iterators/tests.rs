//! Tests for iterators

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use super::*;
use crate::{test_utils::TestLayers, CellMapParams};

/// Check that iterator constructors return the right ok or error.
#[test]
fn construction() {
    // Dummy map
    let mut map = CellMap::<TestLayers, f64>::new_from_elem(
        CellMapParams {
            cell_size: Vector2::new(1.0, 1.0),
            num_cells: Vector2::new(5, 5),
            ..Default::default()
        },
        1.0,
    );

    // Try to build new iterators, checking we don't panic
    let _ = map.iter();
    let _ = map.iter_mut();
    assert!(map.window_iter(Vector2::new(2, 2)).is_ok());
    assert!(map.window_iter_mut(Vector2::new(2, 2)).is_ok());

    // Check that window builds return errors when we use a big window
    assert!(map.window_iter(Vector2::new(3, 3)).is_err());
    assert!(map.window_iter_mut(Vector2::new(3, 3)).is_err());
}

#[test]
fn counts() -> Result<(), CellMapError> {
    // Dummy map
    let mut map = CellMap::<TestLayers, f64>::new_from_elem(
        CellMapParams {
            cell_size: Vector2::new(1.0, 1.0),
            num_cells: Vector2::new(5, 5),
            ..Default::default()
        },
        1.0,
    );

    assert_eq!(map.iter().count(), 75);
    assert_eq!(map.iter().layer(TestLayers::Layer0).count(), 25);
    assert_eq!(
        map.iter()
            .layers(&[TestLayers::Layer0, TestLayers::Layer2])
            .count(),
        50
    );
    assert_eq!(map.iter_mut().count(), 75);
    assert_eq!(map.iter_mut().layer(TestLayers::Layer0).count(), 25);
    assert_eq!(
        map.iter_mut()
            .layers(&[TestLayers::Layer0, TestLayers::Layer2])
            .count(),
        50
    );

    assert_eq!(map.window_iter(Vector2::new(1, 1))?.count(), 27);
    assert_eq!(map.window_iter_mut(Vector2::new(1, 1))?.count(), 27);

    assert_eq!(map.window_iter(Vector2::new(2, 2))?.count(), 3);
    assert_eq!(map.window_iter(Vector2::new(2, 2))?.count(), 3);

    Ok(())
}
