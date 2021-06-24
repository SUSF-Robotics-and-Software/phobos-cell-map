//! Provides the [`CellMapFile`] type which allows a cell map to be serialised using serde.

// ------------------------------------------------------------------------------------------------
// IMPORTS
// ------------------------------------------------------------------------------------------------

use nalgebra::{Affine2, Vector2};
use ndarray::Array2;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{CellMap, CellMapError, CellMapParams, Layer};

// ------------------------------------------------------------------------------------------------
// STRUCTS
// ------------------------------------------------------------------------------------------------

/// Represents a file that can be serialised and deserialised using serde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMapFile<L, T>
where
    L: Layer,
{
    /// Number of layers stored in the map
    pub num_layers: usize,

    /// The order of layers in the map.
    ///
    /// The index of a layer name in this vector matches the index of that layer in the `data`
    /// member.
    pub layers: Vec<L>,

    /// Number of cells in each layer of the map, in the `x` and `y` map-frame directions.
    pub num_cells: Vector2<usize>,

    /// The size of each cell in the map, in parent-frame units.
    pub cell_size: Vector2<f64>,

    /// The precision used when calculating cell boundaries, relative to `cell_size`.
    pub cell_boundary_precision: f64,

    /// The angle which rotates the parent frame into the map frame, in radians.
    pub from_parent_angle_rad: f64,

    /// The translation that goes from the parent frame to the map frame, in parent frame units.
    pub from_parent_translation: Vector2<f64>,

    /// The affine transformation matrix that converts from points in the parent frame to the map frame.
    pub from_parent_matrix: Affine2<f64>,

    /// Stores each layer of the map as an [`ndarray::Array2<T>`].
    pub data: Vec<Array2<T>>,
}

// ------------------------------------------------------------------------------------------------
// IMPLS
// ------------------------------------------------------------------------------------------------

impl<L, T> CellMapFile<L, T>
where
    L: Layer,
{
    /// Converts this file into a [`CellMap`].
    pub fn into_cell_map(self) -> Result<CellMap<L, T>, CellMapError> {
        let params = CellMapParams {
            cell_size: self.cell_size,
            num_cells: self.num_cells,
            rotation_in_parent_rad: self.from_parent_angle_rad,
            position_in_parent: self.from_parent_translation,
            cell_boundary_precision: self.cell_boundary_precision,
        };

        CellMap::new_from_data(params, self.data)
    }
}

impl<L, T> CellMapFile<L, T>
where
    L: Layer + Serialize,
    T: Clone + Serialize,
{
    pub(crate) fn new(map: &CellMap<L, T>) -> Self {
        Self {
            num_layers: L::NUM_LAYERS,
            layers: L::all(),
            num_cells: map.metadata.num_cells,
            cell_size: map.metadata.cell_size,
            cell_boundary_precision: map.metadata.cell_boundary_precision,
            from_parent_angle_rad: map.params.rotation_in_parent_rad,
            from_parent_translation: map.params.position_in_parent,
            from_parent_matrix: map.metadata.to_parent.inverse(),
            data: map.data.clone(),
        }
    }
}

impl<L, T> CellMapFile<L, T>
where
    L: Layer + Serialize,
    T: Serialize,
{
    /// Writes the [`CellMapFile`] to the given path, overwriting any existing file. The format of
    /// the written file is JSON.
    #[cfg(feature = "json")]
    pub fn write_json<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(false)
            .truncate(true)
            .write(true)
            .open(path)?;

        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }
}

impl<L, T> CellMapFile<L, T>
where
    L: Layer + DeserializeOwned,
    T: DeserializeOwned,
{
    /// Loads a [`CellMapFile`] from the given path, which points to a JSON file.
    #[cfg(feature = "json")]
    pub fn from_json<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Open the file
        let file = std::fs::File::open(path)?;
        let map_file: CellMapFile<L, T> = serde_json::from_reader(&file)?;
        Ok(map_file)
    }
}
