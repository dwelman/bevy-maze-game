#![allow(dead_code)]
/// The type of an edge between two grid cells (or a cell boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    /// Solid wall — blocks movement and vision.
    Wall,
    /// No wall — open passage between two cells.
    Open,
    /// Floor surface (Down edges).
    Floor,
    /// Ceiling surface (Up edges).
    Ceiling,
}

/// An edge shared between two adjacent cells (or a cell and the boundary).
/// Each cell owns all 6 of its edges; shared interior edges are mirrored by `Grid::set_edge`.
#[derive(Debug, Clone)]
pub struct Edge {
    pub edge_type: EdgeType,
    /// Whether this edge allows movement through it.
    pub passable: bool,
}

impl Edge {
    /// Solid wall — impassable.
    pub fn wall() -> Self {
        Self { edge_type: EdgeType::Wall, passable: false }
    }

    /// Open passage — passable.
    pub fn open() -> Self {
        Self { edge_type: EdgeType::Open, passable: true }
    }

    /// Floor surface — impassable (player stands on it).
    pub fn floor() -> Self {
        Self { edge_type: EdgeType::Floor, passable: false }
    }

    /// Ceiling surface — impassable.
    pub fn ceiling() -> Self {
        Self { edge_type: EdgeType::Ceiling, passable: false }
    }
}
