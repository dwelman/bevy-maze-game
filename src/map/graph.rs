use bevy::prelude::Resource;
use bevy::math::IVec3;
use std::collections::HashMap;

use crate::map::{Cell, CellId, CardinalDirection};

#[derive(Resource)]
pub struct CellGraph {
    cells: HashMap<CellId, Cell>,
    next_id: u32,
    cell_size: f32,
}

impl CellGraph {
    /// Creates a new empty cell graph with the given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            next_id: 0,
            cell_size,
        }
    }

    /// Returns the configured cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Adds a new cell at the given grid position and returns its ID
    pub fn add_cell(&mut self, position: IVec3) -> CellId {
        let id = CellId(self.next_id);
        self.next_id += 1;

        let cell = Cell::new(id, position);
        self.cells.insert(id, cell);

        id
    }

    /// Returns a reference to the cell with the given ID
    pub fn get_cell(&self, id: CellId) -> Option<&Cell> {
        self.cells.get(&id)
    }

    /// Returns a mutable reference to the cell with the given ID
    pub fn get_cell_mut(&mut self, id: CellId) -> Option<&mut Cell> {
        self.cells.get_mut(&id)
    }

    /// Connects two cells bidirectionally in the given direction
    ///
    /// The connection is made from cell_a in the given direction to cell_b,
    /// and from cell_b in the opposite direction back to cell_a.
    ///
    /// Returns true if both cells exist and were successfully connected.
    pub fn connect_cells(&mut self, cell_a: CellId, direction: CardinalDirection, cell_b: CellId) -> bool {
        // Check that both cells exist
        if !self.cells.contains_key(&cell_a) || !self.cells.contains_key(&cell_b) {
            return false;
        }

        // Connect cell_a to cell_b
        if let Some(cell) = self.cells.get_mut(&cell_a) {
            cell.connect(direction, cell_b);
        }

        // Connect cell_b back to cell_a in the opposite direction
        let opposite = direction.opposite();
        if let Some(cell) = self.cells.get_mut(&cell_b) {
            cell.connect(opposite, cell_a);
        }

        true
    }

    /// Disconnects two cells bidirectionally in the given direction
    ///
    /// Removes the connection from cell_a in the given direction,
    /// and from the connected neighbor back to cell_a.
    ///
    /// Returns true if cell_a exists and had a connection in that direction.
    pub fn disconnect_cells(&mut self, cell_a: CellId, direction: CardinalDirection) -> bool {
        // Get the neighbor ID before disconnecting
        let neighbor_id = match self.cells.get(&cell_a) {
            Some(cell) => cell.get_neighbor(direction),
            None => return false,
        };

        let neighbor_id = match neighbor_id {
            Some(id) => id,
            None => return false,
        };

        // Disconnect cell_a from neighbor
        if let Some(cell) = self.cells.get_mut(&cell_a) {
            cell.disconnect(direction);
        }

        // Disconnect neighbor from cell_a in opposite direction
        let opposite = direction.opposite();
        if let Some(cell) = self.cells.get_mut(&neighbor_id) {
            cell.disconnect(opposite);
        }

        true
    }

    /// Returns an iterator over all cell IDs in the graph
    pub fn cell_ids(&self) -> impl Iterator<Item = CellId> + '_ {
        self.cells.keys().copied()
    }

    /// Returns an iterator over all cells in the graph
    pub fn cells(&self) -> impl Iterator<Item = &Cell> + '_ {
        self.cells.values()
    }

    /// Returns the floor height (bottom Y coordinate) for a given cell
    pub fn get_cell_floor_height(&self, cell_id: CellId) -> Option<f32> {
        self.cells.get(&cell_id).map(|cell| {
            cell.position().y as f32 * self.cell_size - self.cell_size / 2.0
        })
    }
}
