use bevy::math::Vec3;
use std::collections::HashMap;

use crate::map::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId(pub u32);

#[derive(Debug)]
pub struct Cell {
    id: CellId,
    position: Vec3,
    connections: HashMap<Direction, CellId>,
}

impl Cell {
    /// Creates a new cell at the given position
    pub fn new(id: CellId, position: Vec3) -> Self {
        Self {
            id,
            position,
            connections: HashMap::new(),
        }
    }

    /// Returns the cell's ID
    pub fn id(&self) -> CellId {
        self.id
    }

    /// Returns the cell's position in world space
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Connects this cell to a neighbor in the given direction
    pub fn connect(&mut self, direction: Direction, neighbor: CellId) {
        self.connections.insert(direction, neighbor);
    }

    /// Disconnects the neighbor in the given direction, returning the previous neighbor if any
    pub fn disconnect(&mut self, direction: Direction) -> Option<CellId> {
        self.connections.remove(&direction)
    }

    /// Returns the neighbor cell ID in the given direction, if any
    pub fn get_neighbor(&self, direction: Direction) -> Option<CellId> {
        self.connections.get(&direction).copied()
    }

    /// Returns true if this cell has a neighbor in the given direction
    pub fn has_neighbor(&self, direction: Direction) -> bool {
        self.connections.contains_key(&direction)
    }

    /// Returns a list of directions that don't have neighbors (boundary faces)
    pub fn boundary_faces(&self) -> Vec<Direction> {
        Direction::all()
            .into_iter()
            .filter(|dir| !self.has_neighbor(*dir))
            .collect()
    }
}
