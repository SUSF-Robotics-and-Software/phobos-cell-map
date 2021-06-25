# `cell-map`: many-layer 2D cellular maps

[![Crates.io](https://img.shields.io/crates/v/cell-map?style=for-the-badge)](https://crates.io/crates/cell-map) [![docs.rs](https://img.shields.io/docsrs/cell-map?style=for-the-badge)](https://docs.rs/cell-map/)

This crate provides the `CellMap` type, a 2D map with many layers comprised of
cells that can store arbitrary data. It is based on
[ANYbotics/grid_map](https://github.com/ANYbotics/grid_map), a C++ ROS package
which provides the same type of data structre.

`CellMap` uses `ndarray::Array2` to store its data in an efficient and
scalable format. It also uses `nalgebra` types for expressing vectors and
points.

## Getting Started

### Layers

Each layer of the cell map is represented by its own `ndarray::Array2` array.
The map indexes each layer by an enum implementing the `Layer` trait. A derive
macro is provided to simplify this, for example:

```rust
use cell_map::Layer;

#[derive(Layer, Clone, Debug)]
enum MyLayer {
    Height,
    Gradient,
    Roughness
}
```

The `Layer` trait is required to be `Clone`, and is recommended to be `Debug`.

### Creating a `CellMap`

To create a new map:

```rust
use cell_map::{CellMap, CellMapParams, Layer};
use nalgebra::Vector2;

// Creates a new 5x5 map where each cell is 1.0 units wide, which is centred on (0, 0), with
// all elements initialised to 1.0.
let my_map = CellMap::<MyLayer, f64>::new_from_elem(
    CellMapParams {
        cell_size: Vector2::new(1.0, 1.0),
        num_cells: Vector2::new(5, 5),
        centre: Vector2::new(0.0, 0.0),
    },
    1.0,
);
```

### Iterating Over Cells

`CellMap` provides methods to produce iterators over its data:
  - `CellMap::iter()` gives an iterator over all cells in every layer of the map
  - `CellMap::window_iter()` gives an iterator over rectangular windows into
    the map.
  - `CellMap::line_iter()`  gives an iterator of cells between two world points

All iterators also provide a mutable variant, and more iterators are planned
in the future!

You can modify iterators so they produce their indexes, as well as controlling which layers the
data comes from. See `iterators` for more information.

```rust
// Check all the cells in our map are 1, this will be true
assert!(my_map.iter().all(|&v| v == 1.0));

// Use a window iterator to change all cells not on the border of the map to 2
my_map.window_iter_mut(Vector2::new(1, 1)).unwrap().for_each(|mut v| {
    v[(1, 1)] = 2.0;
});

// Overwrite all values on the Roughness layer to be zero
my_map.iter_mut().layer(MyLayer::Roughness).for_each(|v| *v = 0.0);

// Check that our map is how we expect it
for ((layer, cell), &value) in my_map.iter().indexed() {
    if matches!(layer, MyLayer::Roughness) {
        assert_eq!(value, 0.0);
    }
    else if cell.x == 0 || cell.x == 4 || cell.y == 0 || cell.y == 4 {
        assert_eq!(value, 1.0);
    }
    else {
        assert_eq!(value, 2.0);
    }
}
```