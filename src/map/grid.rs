#![allow(dead_code)]
use bevy::prelude::*;
use std::collections::HashMap;

use super::cell::Cell;
use super::direction::Direction;
use super::edge::Edge;

/// The top-level grid resource.
///
/// Cells are keyed by their `IVec3` coordinate. Neighbour links and edge
/// mirroring are kept consistent by the mutation helpers below — prefer
/// `add_cell` / `set_edge` over direct `cells` access.
#[derive(Resource)]
pub struct Grid {
    pub cells: HashMap<IVec3, Cell>,
}

impl Grid {
    /// Create an empty grid.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    /// Insert a new cell at `(x, y, z)`.
    ///
    /// After insertion the cell's `neighbours` map is populated for every
    /// direction in which an adjacent cell already exists, and those neighbours
    /// are updated to point back at the new cell.
    pub fn add_cell(&mut self, x: i32, y: i32, z: i32) {
        let coord = IVec3::new(x, y, z);
        let mut cell = Cell::new(x, y, z);

        // Wire neighbour links in both directions.
        for dir in Direction::all() {
            let neighbour_coord = coord + dir.to_ivec3();
            if self.cells.contains_key(&neighbour_coord) {
                // New cell points at existing neighbour.
                cell.neighbours.insert(dir, Some(neighbour_coord));
                // Existing neighbour points back.
                if let Some(neighbour) = self.cells.get_mut(&neighbour_coord) {
                    neighbour.neighbours.insert(dir.opposite(), Some(coord));
                }
            } else {
                // No neighbour in this direction (boundary).
                cell.neighbours.insert(dir, None);
            }
        }

        self.cells.insert(coord, cell);
    }

    /// Immutable cell lookup.
    pub fn get_cell(&self, coord: IVec3) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    /// Mutable cell lookup.
    pub fn get_cell_mut(&mut self, coord: IVec3) -> Option<&mut Cell> {
        self.cells.get_mut(&coord)
    }

    /// Set the edge on `coord` in `dir` and mirror it onto the neighbour cell
    /// (opposite direction) if one exists.
    ///
    /// This is the canonical way to mutate edges — it keeps both sides in sync.
    pub fn set_edge(&mut self, coord: IVec3, dir: Direction, edge: Edge) {
        // Apply to the source cell.
        if let Some(cell) = self.cells.get_mut(&coord) {
            cell.edges.insert(dir, edge.clone());
        }

        // Mirror to the neighbour if it exists.
        let neighbour_coord = coord + dir.to_ivec3();
        if let Some(neighbour) = self.cells.get_mut(&neighbour_coord) {
            neighbour.edges.insert(dir.opposite(), edge);
        }
    }

    /// Returns `true` if moving from `from` in `dir` is permitted.
    ///
    /// Both conditions must hold:
    /// 1. A cell exists at `from`.
    /// 2. Its edge in `dir` is passable.
    /// 3. A neighbour cell exists in that direction (not a boundary).
    pub fn can_move(&self, from: IVec3, dir: Direction) -> bool {
        match self.cells.get(&from) {
            Some(cell) => {
                cell.is_passable(dir)
                    && cell.neighbours.get(&dir).copied().flatten().is_some()
            }
            None => false,
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}
