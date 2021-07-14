//! Small demo which creates 3 maps, translated, scaled, and rotated, and writes them out if the
//! debug_maps feature is enabled

use cell_map::{Bounds, CellMap, CellMapParams, Layer};
use nalgebra::Vector2;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Layer, Serialize)]
enum Layers {
    Layer0,
    Layer1,
    Layer2,
}

fn main() {
    let translated = CellMap::<Layers, f64>::new(CellMapParams {
        cell_size: Vector2::new(1.0, 1.0),
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
        position_in_parent: Vector2::new(5.0, 5.0),
        ..Default::default()
    });

    let rotated = CellMap::<Layers, f64>::new(CellMapParams {
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
        cell_size: Vector2::new(1.0, 1.0),
        position_in_parent: Vector2::new(5.0, 5.0),
        rotation_in_parent_rad: std::f64::consts::FRAC_PI_4,
        ..Default::default()
    });

    let scaled = CellMap::<Layers, f64>::new(CellMapParams {
        cell_bounds: Bounds::new((0, 10), (0, 10)).unwrap(),
        cell_size: Vector2::new(0.5, 0.5),
        position_in_parent: Vector2::new(5.0, 5.0),
        rotation_in_parent_rad: std::f64::consts::FRAC_PI_4,
        ..Default::default()
    });

    #[cfg(all(feature = "debug_maps"))]
    {
        use cell_map::write_debug_map;

        write_debug_map(&translated, "translated");
        write_debug_map(&rotated, "rotated");
        write_debug_map(&scaled, "scaled");
    }
}
