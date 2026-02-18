#![allow(dead_code)]
use bevy::prelude::*;
use std::collections::HashMap;

use super::direction::Direction;
use super::edge::Edge;

/// A single cell in the 3-D grid.
///
/// Each cell owns all 6 of its edges directly. Interior edges are kept in sync
/// with neighbours by `Grid::set_edge`, which mirrors every change to the
/// opposite face of the adjacent cell.
#[derive(Debug, Clone)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// All 6 edges owned by this cell (one per direction).
    pub edges: HashMap<Direction, Edge>,
    /// Coordinates of each neighbour — `None` means grid boundary.
    pub neighbours: HashMap<Direction, Option<IVec3>>,
}

impl Cell {
    /// Create a new cell with all 6 edges initialised to `Edge::wall()`.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let mut edges = HashMap::new();
        for dir in Direction::all() {
            edges.insert(dir, Edge::wall());
        }
        Self {
            x,
            y,
            z,
            edges,
            neighbours: HashMap::new(),
        }
    }

    pub fn coords(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    pub fn get_edge(&self, dir: Direction) -> &Edge {
        self.edges.get(&dir).expect("all 6 edges should always be present")
    }

    pub fn get_edge_mut(&mut self, dir: Direction) -> &mut Edge {
        self.edges.get_mut(&dir).expect("all 6 edges should always be present")
    }

    /// Returns `true` if movement through `dir` is permitted by the edge.
    pub fn is_passable(&self, dir: Direction) -> bool {
        self.get_edge(dir).passable
    }
}
