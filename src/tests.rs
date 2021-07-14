//! Tests of [`CellMap`] level functionality.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Point2, Vector2};

use super::*;
use crate::{cell_map::Bounds, test_utils::TestLayers};

// ------------------------------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------------------------------

#[test]
fn get_cell_positions() {
    // Empty map with no difference to the parent
    let map = CellMap::<TestLayers, f64>::new(CellMapParams {
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
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
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
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
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
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
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
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

#[test]
fn test_resize() {
    let mut map = CellMap::<TestLayers, Option<i32>>::new_from_elem(
        CellMapParams {
            cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
            cell_size: Vector2::new(1.0, 1.0),
            ..Default::default()
        },
        Some(1),
    );

    // Resize it with an extra 5 cells on the border
    let new_bounds = Bounds::new((-5, 15), (-5, 15)).unwrap();
    map.resize(new_bounds);

    // Check shape related data
    assert_eq!(map.cell_bounds(), new_bounds);
    assert_eq!(map.num_cells(), Vector2::new(20, 20));

    // Check that the borders are None, but the old map area is Some
    for ((_, idx), &val) in map.iter().indexed().layer(TestLayers::Layer0) {
        if idx.x < 5 || idx.x >= 15 || idx.y < 5 || idx.y >= 15 {
            assert_eq!(val, None);
        } else {
            assert_eq!(val, Some(1));
        }
    }

    // Resize it so we cut off some of the known data
    let new_bounds = Bounds::new((8, 12), (-5, 15)).unwrap();
    map.resize(new_bounds);

    // Check shape related data
    assert_eq!(map.cell_bounds(), new_bounds);
    assert_eq!(map.num_cells(), Vector2::new(4, 20));

    // Check that the borders are None, but the old map area is Some
    for ((_, idx), &val) in map.iter().indexed().layer(TestLayers::Layer0) {
        if idx.x >= 2 || idx.y < 5 || idx.y >= 15 {
            assert_eq!(val, None);
        } else {
            assert_eq!(val, Some(1));
        }
    }
}

#[test]
fn test_merge() {
    let mut map_a = CellMap::<TestLayers, i32>::new_from_elem(
        CellMapParams {
            cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
            cell_size: Vector2::new(1.0, 1.0),
            ..Default::default()
        },
        1,
    );

    #[cfg(feature = "debug_maps")]
    crate::write_debug_map(&map_a, "a");

    // Print the map
    let mut last_y = 0;
    print!("\nA:\n    ");
    for ((_, idx), val) in map_a.iter().layer(TestLayers::Layer0).indexed() {
        if last_y != idx.y {
            last_y = idx.y;
            print!("\n    ");
        }

        print!("{} ", val);
    }
    println!();

    let map_b = CellMap::<TestLayers, i32>::new_from_elem(
        CellMapParams {
            cell_bounds: Bounds::new((5, 15), (5, 15)).unwrap(),
            cell_size: Vector2::new(0.5, 0.5),
            // position_in_parent: Vector2::new(5.0, 5.0),
            rotation_in_parent_rad: std::f64::consts::FRAC_PI_4,
            ..Default::default()
        },
        2,
    );

    #[cfg(feature = "debug_maps")]
    crate::write_debug_map(&map_b, "b");

    let mut last_y = 0;
    print!("\nB:\n    ");
    for ((_, idx), val) in map_b.iter().layer(TestLayers::Layer0).indexed() {
        if last_y != idx.y {
            last_y = idx.y;
            print!("\n    ");
        }

        print!("{} ", val);
    }
    println!();

    // Simple average merge
    map_a.merge(&map_b, |&a, bs| {
        let mut acc = a;
        for &b in bs {
            acc += b
        }

        (acc as f64 / (bs.len() as f64 + 1.0)).round() as i32
    });

    #[cfg(feature = "debug_maps")]
    crate::write_debug_map(&map_a, "merged");

    // Print the map
    let mut last_y = 0;
    print!("\nA + B:\n    ");
    for ((_, idx), val) in map_a.iter().layer(TestLayers::Layer0).indexed() {
        if last_y != idx.y {
            last_y = idx.y;
            print!("\n    ");
        }

        print!("{} ", val);
    }
    println!();
}
