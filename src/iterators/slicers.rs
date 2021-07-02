//! Provides the [`Slicer`] trait and slicers types that determine the order and pattern in which
//! data is produced by an interator over a [`CellMap`].
//!
//! [`CellMap`]: crate::CellMap

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Point2, Vector2};
use ndarray::{s, Array2, ArrayView2, ArrayViewMut2};
use serde::Serialize;

use crate::{extensions::Point2Ext, map_metadata::CellMapMetadata, CellMap, Error, Layer};

// ------------------------------------------------------------------------------------------------
// TRAITS
// ------------------------------------------------------------------------------------------------

/// Trait which allows a [`CellMapIter`] or [`CellMapIterMut`] struct to determine what shape the
/// data in the iteration should be produced in. For example:
///
/// - [`Cells`] produces data in each cell in `(x, y)` order, x increasing most rapidly.
/// - [`Windows`] produces rectangular views in `(x, y`) order, x increasing most rapidly.
/// - [`Line`] produces cells along the line connecting two positions in the parent frame.
///
/// [`Slicer`]s are designed to be used in iterators, so after a `.slice` or `.slice_mut` the user
/// shall call [`Slicer::advance()`] on the type.
///
/// [`CellMapIter`]: crate::iterators::CellMapIter
/// [`CellMapIterMut`]: crate::iterators::CellMapIterMut
pub trait Slicer<'a, L, T>
where
    L: Layer,
{
    /// The non-mutable output type for the data of this [`Slicer`].
    type Output;

    /// The mutable output type for the data of this [`Slicer`].
    type OutputMut;

    /// Perform the slice on the given `data` layer, or `None` if the slicer has reached the end of
    /// its data.
    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output>;

    /// Perform a mutable slice on the given `data` layer, or `None` if the slicer has reached the
    /// end of its data.
    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut>;

    /// Advance the [`Slicer`] to the next index.
    fn advance(&mut self);

    /// Return the current index of the [`Slicer`], or `None` if the slicer has reached the end of
    /// its data.
    fn index(&self) -> Option<Point2<usize>>;

    /// Resets the index of this [`Slicer`] so that it can be used on the next layer in the
    /// iteration. The `layer` input is used for slicers which need to monitor which layer they are
    /// on.
    fn reset(&mut self, layer: Option<L>);
}

/// Rectangular bounds in an XY plane. Lower bound is inclusive, upper exclusive.
pub(crate) type RectBounds = Vector2<(usize, usize)>;

/// A [`Slicer`] which produces cells in `(x, y)` order inside a layer, with `x` increasing most
/// rapidly.
#[derive(Debug, Clone, Copy)]
pub struct Cells {
    bounds: RectBounds,
    index: Point2<usize>,
}

/// A [`Slicer`] which produces rectangular views into a layer in `(x, y)` order, increasing `x`
/// most rapidly. A boundary of the `semi_width` of the window around the outside edge of the map
/// is used to prevent indexing outside the map.
#[derive(Debug, Clone, Copy)]
pub struct Windows {
    bounds: RectBounds,
    index: Point2<usize>,
    semi_width: Vector2<usize>,
}

/// A [`Slicer`] which produces cells along the line connecting two points in the parent frame.
///
/// This slicer will uses the algorithm described in
/// [here](https://theshoemaker.de/2016/02/ray-casting-in-2d-grids/), meaning that all cells the
/// line intersects will be yielded only once.
#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct Line {
    bounds: RectBounds,
    map_meta: CellMapMetadata,

    start_parent: Point2<f64>,
    end_parent: Point2<f64>,

    dir: Vector2<f64>,
    dir_sign: Vector2<f64>,

    start_map: Point2<f64>,
    end_map: Point2<f64>,
    current_map: Option<Point2<f64>>,

    end_index: Point2<usize>,

    #[cfg(feature = "debug_iters")]
    step_report_file: std::sync::Arc<std::fs::File>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct LineStepData {
    start_parent: Point2<f64>,
    end_parent: Point2<f64>,

    dir: Vector2<f64>,
    dir_sign: Vector2<f64>,

    start_map: Point2<f64>,
    end_map: Point2<f64>,
    current_map: Option<Point2<f64>>,

    end_index: Point2<usize>,

    delta: Vector2<f64>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl Cells {
    pub(crate) fn from_map<L: Layer, T>(map: &CellMap<L, T>) -> Self {
        let cells = map.num_cells();
        Self {
            bounds: Vector2::new((0, cells.x), (0, cells.y)),
            index: Point2::new(0, 0),
        }
    }
}

impl<'a, L, T> Slicer<'a, L, T> for Cells
where
    L: Layer,
    T: 'a,
{
    type Output = &'a T;
    type OutputMut = &'a mut T;

    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output> {
        data.get(self.index.as_array2_index())
    }

    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut> {
        data.get_mut(self.index.as_array2_index())
    }

    fn advance(&mut self) {
        self.index.x += 1;

        if !self.index.in_bounds(&self.bounds) {
            self.index.y += 1;
            self.index.x = self.bounds.x.0;
        }
    }

    fn index(&self) -> Option<Point2<usize>> {
        if self.index.in_bounds(&self.bounds) {
            Some(self.index)
        } else {
            None
        }
    }

    fn reset(&mut self, _layer: Option<L>) {
        self.index = Point2::new(self.bounds.x.0, self.bounds.y.0);
    }
}

impl Windows {
    pub(crate) fn from_map<L: Layer, T>(
        map: &CellMap<L, T>,
        semi_width: Vector2<usize>,
    ) -> Result<Self, Error> {
        let cells = map.num_cells();

        if semi_width.x * 2 + 1 > cells.x || semi_width.y * 2 + 1 > cells.y {
            Err(Error::WindowLargerThanMap(
                semi_width * 2 + Vector2::new(1, 1),
                cells,
            ))
        } else {
            let bounds = Vector2::new(
                (semi_width.x, cells.x - semi_width.x),
                (semi_width.y, cells.y - semi_width.y),
            );

            Ok(Self {
                bounds,
                index: Point2::new(bounds.x.0, bounds.y.0),
                semi_width,
            })
        }
    }
}

impl<'a, L, T> Slicer<'a, L, T> for Windows
where
    L: Layer,
    T: 'a,
{
    type Output = ArrayView2<'a, T>;
    type OutputMut = ArrayViewMut2<'a, T>;

    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output> {
        if self.index.in_bounds(&self.bounds) {
            let x0 = self.index.x - self.semi_width.x;
            let x1 = self.index.x + self.semi_width.x + 1;
            let y0 = self.index.y - self.semi_width.y;
            let y1 = self.index.y + self.semi_width.y + 1;
            Some(data.slice(s![y0..y1, x0..x1]))
        } else {
            None
        }
    }

    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut> {
        if self.index.in_bounds(&self.bounds) {
            let x0 = self.index.x - self.semi_width.x;
            let x1 = self.index.x + self.semi_width.x + 1;
            let y0 = self.index.y - self.semi_width.y;
            let y1 = self.index.y + self.semi_width.y + 1;
            Some(data.slice_mut(s![y0..y1, x0..x1]))
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.index.x += 1;

        if !self.index.in_bounds(&self.bounds) {
            self.index.y += 1;
            self.index.x = self.bounds.x.0;
        }
    }

    fn index(&self) -> Option<Point2<usize>> {
        if self.index.in_bounds(&self.bounds) {
            Some(self.index)
        } else {
            None
        }
    }

    fn reset(&mut self, _layer: Option<L>) {
        self.index = Point2::new(self.bounds.x.0, self.bounds.y.0);
    }
}

impl Line {
    pub(crate) fn from_map<L: Layer, T>(
        map_meta: CellMapMetadata,
        start_parent: Point2<f64>,
        end_parent: Point2<f64>,
    ) -> Result<Self, Error> {
        // Calculate start and end points in map frame, note these aren't cell indices, instead
        // they are floating point positions within the map frame, which we get by not casting the
        // output of the `to_parent` transforms to usize.
        let start_map = map_meta.to_parent.inverse_transform_point(&start_parent);
        let end_map = map_meta.to_parent.inverse_transform_point(&end_parent);

        // Get map edges in floating point for bounds check
        let map_x_lim = (map_meta.num_cells.x) as f64;
        let map_y_lim = (map_meta.num_cells.y) as f64;

        // Check start and end points are inside the map
        if start_map.x < 0.0
            || start_map.x > map_x_lim
            || start_map.y < 0.0
            || start_map.y > map_y_lim
        {
            return Err(Error::PositionOutsideMap(
                "Line::Start".into(),
                start_parent,
            ));
        }

        if end_map.x < 0.0 || end_map.x > map_x_lim || end_map.y < 0.0 || end_map.y > map_y_lim {
            return Err(Error::PositionOutsideMap("Line::End".into(), start_parent));
        }

        // Calculate direction vector
        let dir = end_map - start_map;

        // Get the direction sign
        let dir_sign = dir.map(|v| if v < 0.0 { 0.0 } else { 1.0 });

        // Get the cell index of the end point
        let end_cell = map_meta
            .index(end_parent)
            .ok_or_else(|| Error::PositionOutsideMap("Line::End".into(), end_parent))?;

        Ok(Self {
            bounds: map_meta.get_bounds(),
            map_meta,
            start_parent,
            end_parent,
            dir,
            dir_sign,
            start_map,
            end_map,
            current_map: Some(start_map),
            end_index: end_cell,
            #[cfg(feature = "debug_iters")]
            step_report_file: std::sync::Arc::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("line_step_report.json")
                    .unwrap(),
            ),
        })
    }

    /// Gets the current cell index to yield, or `None` if at the end of the line
    fn get_current_index(&self) -> Option<Point2<usize>> {
        // Current will be inside the map, since start and end were confirmed to be inside the map
        // at construction, so simply cast
        Some(self.current_map?.map(|v| v as usize))
    }
}

impl<'a, L, T> Slicer<'a, L, T> for Line
where
    L: Layer,
    T: 'a,
{
    type Output = &'a T;

    type OutputMut = &'a mut T;

    fn slice(&self, data: &'a Array2<T>) -> Option<Self::Output> {
        // Get the index
        let index = self.get_current_index()?;

        data.get(index.as_array2_index())
    }

    fn slice_mut(&self, data: &'a mut Array2<T>) -> Option<Self::OutputMut> {
        // Get the index
        let index = self.get_current_index()?;

        data.get_mut(index.as_array2_index())
    }

    fn advance(&mut self) {
        // Get the index of the current position, or just return if we're at the end
        let curr_index = match self.get_current_index() {
            Some(i) => i,
            None => return,
        };

        // Calculate the param value, i.e. how far along the line we are. If it > 1 we're at the end
        let param = (self.current_map.unwrap() - self.start_map).norm()
            / (self.end_map - self.start_map).norm();
        if param > 1.0 {
            self.current_map = None;
            return;
        }

        // // If the current index matches the end cell, we are at the end, and set current to None
        // if curr_index == self.end_index {
        //     self.current_map = None;
        //     return;
        // }

        // Calculate the changes in the line parameter needed to reach the next x and y grid line
        // respectively. Also add on the cell boundary precision to ensure that we will actually
        // move over the cell boundary line.
        let delta_param: Vector2<f64> = ((curr_index.cast() + self.dir_sign)
            - self.current_map.unwrap())
        .component_div(&self.dir)
            + Vector2::from_element(self.map_meta.cell_boundary_precision);

        // Whichever component of delta is smaller is what we need to advance along the line by.
        let delta = delta_param.x.min(delta_param.y);

        // Move current to current + dir*delta
        self.current_map = Some(self.current_map.unwrap() + (self.dir * delta));

        // Write new step report to file
        #[cfg(feature = "debug_iters")]
        {
            use std::io::Write;

            let rpt = LineStepData {
                start_parent: self.start_parent,
                end_parent: self.end_parent,
                dir: self.dir,
                dir_sign: self.dir_sign,
                start_map: self.start_map,
                end_map: self.end_map,
                current_map: self.current_map,
                end_index: self.end_index,
                delta: delta_param,
            };

            let val = serde_json::to_string_pretty(&rpt).unwrap();

            writeln!(
                std::sync::Arc::<std::fs::File>::get_mut(&mut self.step_report_file).unwrap(),
                "{},",
                val
            )
            .unwrap();
        }
    }

    fn index(&self) -> Option<Point2<usize>> {
        self.get_current_index()
    }

    fn reset(&mut self, _layer: Option<L>) {
        self.current_map = Some(self.start_map)
    }
}
