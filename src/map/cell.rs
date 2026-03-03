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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell(id: u32, x: f32, y: f32, z: f32) -> Cell {
        Cell::new(CellId(id), Vec3::new(x, y, z))
    }

    // --- Cell::new / id / position ---

    #[test]
    fn new_stores_id_and_position() {
        let cell = make_cell(1, 1.0, 2.0, 3.0);
        assert_eq!(cell.id(), CellId(1));
        assert_eq!(cell.position(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn new_starts_with_no_connections() {
        let cell = make_cell(1, 0.0, 0.0, 0.0);
        for dir in Direction::all() {
            assert!(!cell.has_neighbor(dir));
        }
    }

    #[test]
    fn new_zero_position() {
        let cell = make_cell(0, 0.0, 0.0, 0.0);
        assert_eq!(cell.position(), Vec3::ZERO);
    }

    #[test]
    fn new_negative_position() {
        let cell = make_cell(5, -10.0, -20.0, -30.0);
        assert_eq!(cell.position(), Vec3::new(-10.0, -20.0, -30.0));
    }

    // --- Cell::id ---

    #[test]
    fn id_returns_correct_value() {
        let cell = make_cell(42, 0.0, 0.0, 0.0);
        assert_eq!(cell.id(), CellId(42));
    }

    #[test]
    fn id_zero() {
        let cell = make_cell(0, 0.0, 0.0, 0.0);
        assert_eq!(cell.id(), CellId(0));
    }

    #[test]
    fn id_max_u32() {
        let cell = Cell::new(CellId(u32::MAX), Vec3::ZERO);
        assert_eq!(cell.id(), CellId(u32::MAX));
    }

    // --- Cell::position ---

    #[test]
    fn position_returns_correct_vec3() {
        let cell = make_cell(1, 5.5, -3.0, 0.25);
        assert_eq!(cell.position(), Vec3::new(5.5, -3.0, 0.25));
    }

    // --- Cell::connect ---

    #[test]
    fn connect_creates_neighbor() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::North, CellId(2));
        assert_eq!(cell.get_neighbor(Direction::North), Some(CellId(2)));
    }

    #[test]
    fn connect_overwrites_existing_neighbor() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::North, CellId(2));
        cell.connect(Direction::North, CellId(99));
        assert_eq!(cell.get_neighbor(Direction::North), Some(CellId(99)));
    }

    #[test]
    fn connect_all_directions() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        let neighbors = [
            (Direction::North, CellId(2)),
            (Direction::South, CellId(3)),
            (Direction::East, CellId(4)),
            (Direction::West, CellId(5)),
            (Direction::Up, CellId(6)),
            (Direction::Down, CellId(7)),
        ];
        for (dir, id) in neighbors {
            cell.connect(dir, id);
        }
        for (dir, id) in neighbors {
            assert_eq!(cell.get_neighbor(dir), Some(id));
        }
    }

    // --- Cell::disconnect ---

    #[test]
    fn disconnect_returns_previous_neighbor() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::South, CellId(10));
        let removed = cell.disconnect(Direction::South);
        assert_eq!(removed, Some(CellId(10)));
    }

    #[test]
    fn disconnect_removes_connection() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::South, CellId(10));
        cell.disconnect(Direction::South);
        assert!(!cell.has_neighbor(Direction::South));
    }

    #[test]
    fn disconnect_empty_direction_returns_none() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        let removed = cell.disconnect(Direction::East);
        assert_eq!(removed, None);
    }

    #[test]
    fn disconnect_then_reconnect() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::West, CellId(5));
        cell.disconnect(Direction::West);
        cell.connect(Direction::West, CellId(99));
        assert_eq!(cell.get_neighbor(Direction::West), Some(CellId(99)));
    }

    // --- Cell::get_neighbor ---

    #[test]
    fn get_neighbor_returns_some_when_connected() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::Up, CellId(8));
        assert_eq!(cell.get_neighbor(Direction::Up), Some(CellId(8)));
    }

    #[test]
    fn get_neighbor_returns_none_when_unconnected() {
        let cell = make_cell(1, 0.0, 0.0, 0.0);
        assert_eq!(cell.get_neighbor(Direction::Down), None);
    }

    #[test]
    fn get_neighbor_unaffected_by_other_directions() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::North, CellId(2));
        assert_eq!(cell.get_neighbor(Direction::South), None);
        assert_eq!(cell.get_neighbor(Direction::East), None);
    }

    // --- Cell::has_neighbor ---

    #[test]
    fn has_neighbor_true_when_connected() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::East, CellId(3));
        assert!(cell.has_neighbor(Direction::East));
    }

    #[test]
    fn has_neighbor_false_when_not_connected() {
        let cell = make_cell(1, 0.0, 0.0, 0.0);
        assert!(!cell.has_neighbor(Direction::West));
    }

    #[test]
    fn has_neighbor_false_after_disconnect() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::Up, CellId(6));
        cell.disconnect(Direction::Up);
        assert!(!cell.has_neighbor(Direction::Up));
    }

    // --- Cell::boundary_faces ---

    #[test]
    fn boundary_faces_all_directions_when_no_connections() {
        let cell = make_cell(1, 0.0, 0.0, 0.0);
        let faces: std::collections::HashSet<Direction> = cell.boundary_faces().into_iter().collect();
        let expected: std::collections::HashSet<Direction> = Direction::all().into_iter().collect();
        assert_eq!(faces, expected);
    }

    #[test]
    fn boundary_faces_excludes_connected_directions() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        cell.connect(Direction::North, CellId(2));
        cell.connect(Direction::East, CellId(3));
        let faces = cell.boundary_faces();
        assert!(!faces.contains(&Direction::North));
        assert!(!faces.contains(&Direction::East));
        assert_eq!(faces.len(), 4);
    }

    #[test]
    fn boundary_faces_empty_when_all_connected() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        for (i, dir) in Direction::all().into_iter().enumerate() {
            cell.connect(dir, CellId(i as u32 + 2));
        }
        assert!(cell.boundary_faces().is_empty());
    }

    #[test]
    fn boundary_faces_updates_after_disconnect() {
        let mut cell = make_cell(1, 0.0, 0.0, 0.0);
        for (i, dir) in Direction::all().into_iter().enumerate() {
            cell.connect(dir, CellId(i as u32 + 2));
        }
        cell.disconnect(Direction::Down);
        let faces = cell.boundary_faces();
        assert_eq!(faces, vec![Direction::Down]);
    }
}
