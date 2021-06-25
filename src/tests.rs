//! Tests of [`CellMap`] level functionality.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Point2, Vector2};

use super::*;
use crate::test_utils::TestLayers;

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[test]
fn get_cell_positions() {
    // Empty map with no difference to the parent
    let map = CellMap::<TestLayers, f64>::new(CellMapParams {
        num_cells: Vector2::new(10, 10),
        cell_size: Vector2::new(1.0, 1.0),
        ..Default::default()
    });

    // Check positions
    assert_f64_iter_eq!(
        map.position(Point2::new(0, 0)).unwrap(),
        Point2::new(0.5, 0.5)
    );
    assert_f64_iter_eq!(
        map.position(Point2::new(5, 5)).unwrap(),
        Point2::new(5.5, 5.5)
    );

    // Check indexes
    assert_eq!(map.index(Point2::new(0.7, 0.1)).unwrap(), Point2::new(0, 0));
    assert_eq!(map.index(Point2::new(7.0, 1.0)).unwrap(), Point2::new(7, 1));
    assert_eq!(
        map.index(Point2::new(2.6, 3.999999)).unwrap(),
        Point2::new(2, 3)
    );
    assert_eq!(map.index(Point2::new(2.6, 4.0)).unwrap(), Point2::new(2, 4));

    // Empty map with scaling
    let map = CellMap::<TestLayers, f64>::new(CellMapParams {
        num_cells: Vector2::new(10, 10),
        cell_size: Vector2::new(0.1, 0.1),
        ..Default::default()
    });

    // Check positions
    assert_f64_iter_eq!(
        map.position(Point2::new(0, 0)).unwrap(),
        Point2::new(0.05, 0.05)
    );
    assert_f64_iter_eq!(
        map.position(Point2::new(5, 5)).unwrap(),
        Point2::new(0.55, 0.55)
    );

    // Check indexes
    assert_eq!(map.index(Point2::new(0.7, 0.1)).unwrap(), Point2::new(7, 1));
    assert_eq!(
        map.index(Point2::new(0.26, 0.3999999)).unwrap(),
        Point2::new(2, 3)
    );
    assert_eq!(
        map.index(Point2::new(0.26, 0.4)).unwrap(),
        Point2::new(2, 4)
    );

    // Empty map with scaling and translation
    let map = CellMap::<TestLayers, f64>::new(CellMapParams {
        num_cells: Vector2::new(10, 10),
        cell_size: Vector2::new(0.1, 0.1),
        position_in_parent: Vector2::new(0.5, 0.5),
        ..Default::default()
    });

    // Check positions
    assert_f64_iter_eq!(
        map.position(Point2::new(0, 0)).unwrap(),
        Point2::new(0.55, 0.55)
    );
    assert_f64_iter_eq!(
        map.position(Point2::new(5, 5)).unwrap(),
        Point2::new(1.05, 1.05)
    );

    // Check indexes
    assert_eq!(map.index(Point2::new(0.7, 0.6)).unwrap(), Point2::new(2, 1));
    assert_eq!(
        map.index(Point2::new(0.76, 0.8999999)).unwrap(),
        Point2::new(2, 3)
    );
    assert_eq!(
        map.index(Point2::new(0.76, 0.9)).unwrap(),
        Point2::new(2, 4)
    );

    // Empty map with scaling, translation and rotation (by pi/2 rad)
    let map = CellMap::<TestLayers, f64>::new(CellMapParams {
        num_cells: Vector2::new(10, 10),
        cell_size: Vector2::new(0.1, 0.1),
        position_in_parent: Vector2::new(0.5, 0.5),
        rotation_in_parent_rad: std::f64::consts::FRAC_PI_4,
        ..Default::default()
    });

    #[cfg(feature = "debug_maps")]
    crate::write_debug_map(&map, "trs");

    // Check positions
    assert_f64_iter_eq!(
        map.position(Point2::new(0, 0)).unwrap(),
        Point2::new(0.5, 0.5707106781186547)
    );
    assert_f64_iter_eq!(
        map.position(Point2::new(5, 5)).unwrap(),
        Point2::new(0.5, 1.2778174593052023)
    );

    // Check indexes
    assert_eq!(map.index(Point2::new(0.4, 0.7)).unwrap(), Point2::new(0, 2));
    assert_eq!(map.index(Point2::new(1.0, 1.2)).unwrap(), Point2::new(8, 1));
    assert_eq!(
        map.index(Point2::new(-0.1, 1.2)).unwrap(),
        Point2::new(0, 9)
    );
}
