use bevy::math::IVec3;
use std::collections::HashMap;

use crate::map::CardinalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId(pub u32);

#[derive(Debug)]
pub struct Cell {
    id: CellId,
    position: IVec3,
    connections: HashMap<CardinalDirection, CellId>,
}

impl Cell {
    /// Creates a new cell at the given grid position
    pub fn new(id: CellId, position: IVec3) -> Self {
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

    /// Returns the cell's position in grid coordinates
    pub fn position(&self) -> IVec3 {
        self.position
    }

    /// Connects this cell to a neighbor in the given direction
    pub fn connect(&mut self, direction: CardinalDirection, neighbor: CellId) {
        self.connections.insert(direction, neighbor);
    }

    /// Disconnects the neighbor in the given direction, returning the previous neighbor if any
    pub fn disconnect(&mut self, direction: CardinalDirection) -> Option<CellId> {
        self.connections.remove(&direction)
    }

    /// Returns the neighbor cell ID in the given direction, if any
    pub fn get_neighbor(&self, direction: CardinalDirection) -> Option<CellId> {
        self.connections.get(&direction).copied()
    }

    /// Returns true if this cell has a neighbor in the given direction
    pub fn has_neighbor(&self, direction: CardinalDirection) -> bool {
        self.connections.contains_key(&direction)
    }

    /// Returns a list of directions that don't have neighbors (boundary faces)
    pub fn boundary_faces(&self) -> Vec<CardinalDirection> {
        CardinalDirection::all()
            .into_iter()
            .filter(|dir| !self.has_neighbor(*dir))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell(id: u32, x: i32, y: i32, z: i32) -> Cell {
        Cell::new(CellId(id), IVec3::new(x, y, z))
    }

    // --- Cell::new / id / position ---

    #[test]
    fn new_stores_id_and_position() {
        let cell = make_cell(1, 1, 2, 3);
        assert_eq!(cell.id(), CellId(1));
        assert_eq!(cell.position(), IVec3::new(1, 2, 3));
    }

    #[test]
    fn new_starts_with_no_connections() {
        let cell = make_cell(1, 0, 0, 0);
        for dir in CardinalDirection::all() {
            assert!(!cell.has_neighbor(dir));
        }
    }

    #[test]
    fn new_zero_position() {
        let cell = make_cell(0, 0, 0, 0);
        assert_eq!(cell.position(), IVec3::ZERO);
    }

    #[test]
    fn new_negative_position() {
        let cell = make_cell(5, -10, -20, -30);
        assert_eq!(cell.position(), IVec3::new(-10, -20, -30));
    }

    // --- Cell::id ---

    #[test]
    fn id_returns_correct_value() {
        let cell = make_cell(42, 0, 0, 0);
        assert_eq!(cell.id(), CellId(42));
    }

    #[test]
    fn id_zero() {
        let cell = make_cell(0, 0, 0, 0);
        assert_eq!(cell.id(), CellId(0));
    }

    #[test]
    fn id_max_u32() {
        let cell = Cell::new(CellId(u32::MAX), IVec3::ZERO);
        assert_eq!(cell.id(), CellId(u32::MAX));
    }

    // --- Cell::position ---

    #[test]
    fn position_returns_correct_ivec3() {
        let cell = make_cell(1, 5, -3, 0);
        assert_eq!(cell.position(), IVec3::new(5, -3, 0));
    }

    // --- Cell::connect ---

    #[test]
    fn connect_creates_neighbor() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::North, CellId(2));
        assert_eq!(cell.get_neighbor(CardinalDirection::North), Some(CellId(2)));
    }

    #[test]
    fn connect_overwrites_existing_neighbor() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::North, CellId(2));
        cell.connect(CardinalDirection::North, CellId(99));
        assert_eq!(cell.get_neighbor(CardinalDirection::North), Some(CellId(99)));
    }

    #[test]
    fn connect_all_directions() {
        let mut cell = make_cell(1, 0, 0, 0);
        let neighbors = [
            (CardinalDirection::North, CellId(2)),
            (CardinalDirection::South, CellId(3)),
            (CardinalDirection::East, CellId(4)),
            (CardinalDirection::West, CellId(5)),
            (CardinalDirection::Zenith, CellId(6)),
            (CardinalDirection::Nadir, CellId(7)),
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
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::South, CellId(10));
        let removed = cell.disconnect(CardinalDirection::South);
        assert_eq!(removed, Some(CellId(10)));
    }

    #[test]
    fn disconnect_removes_connection() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::South, CellId(10));
        cell.disconnect(CardinalDirection::South);
        assert!(!cell.has_neighbor(CardinalDirection::South));
    }

    #[test]
    fn disconnect_empty_direction_returns_none() {
        let mut cell = make_cell(1, 0, 0, 0);
        let removed = cell.disconnect(CardinalDirection::East);
        assert_eq!(removed, None);
    }

    #[test]
    fn disconnect_then_reconnect() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::West, CellId(5));
        cell.disconnect(CardinalDirection::West);
        cell.connect(CardinalDirection::West, CellId(99));
        assert_eq!(cell.get_neighbor(CardinalDirection::West), Some(CellId(99)));
    }

    // --- Cell::get_neighbor ---

    #[test]
    fn get_neighbor_returns_some_when_connected() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::Zenith, CellId(8));
        assert_eq!(cell.get_neighbor(CardinalDirection::Zenith), Some(CellId(8)));
    }

    #[test]
    fn get_neighbor_returns_none_when_unconnected() {
        let cell = make_cell(1, 0, 0, 0);
        assert_eq!(cell.get_neighbor(CardinalDirection::Nadir), None);
    }

    #[test]
    fn get_neighbor_unaffected_by_other_directions() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::North, CellId(2));
        assert_eq!(cell.get_neighbor(CardinalDirection::South), None);
        assert_eq!(cell.get_neighbor(CardinalDirection::East), None);
    }

    // --- Cell::has_neighbor ---

    #[test]
    fn has_neighbor_true_when_connected() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::East, CellId(3));
        assert!(cell.has_neighbor(CardinalDirection::East));
    }

    #[test]
    fn has_neighbor_false_when_not_connected() {
        let cell = make_cell(1, 0, 0, 0);
        assert!(!cell.has_neighbor(CardinalDirection::West));
    }

    #[test]
    fn has_neighbor_false_after_disconnect() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::Zenith, CellId(6));
        cell.disconnect(CardinalDirection::Zenith);
        assert!(!cell.has_neighbor(CardinalDirection::Zenith));
    }

    // --- Cell::boundary_faces ---

    #[test]
    fn boundary_faces_all_directions_when_no_connections() {
        let cell = make_cell(1, 0, 0, 0);
        let faces: std::collections::HashSet<CardinalDirection> = cell.boundary_faces().into_iter().collect();
        let expected: std::collections::HashSet<CardinalDirection> = CardinalDirection::all().into_iter().collect();
        assert_eq!(faces, expected);
    }

    #[test]
    fn boundary_faces_excludes_connected_directions() {
        let mut cell = make_cell(1, 0, 0, 0);
        cell.connect(CardinalDirection::North, CellId(2));
        cell.connect(CardinalDirection::East, CellId(3));
        let faces = cell.boundary_faces();
        assert!(!faces.contains(&CardinalDirection::North));
        assert!(!faces.contains(&CardinalDirection::East));
        assert_eq!(faces.len(), 4);
    }

    #[test]
    fn boundary_faces_empty_when_all_connected() {
        let mut cell = make_cell(1, 0, 0, 0);
        for (i, dir) in CardinalDirection::all().into_iter().enumerate() {
            cell.connect(dir, CellId(i as u32 + 2));
        }
        assert!(cell.boundary_faces().is_empty());
    }

    #[test]
    fn boundary_faces_updates_after_disconnect() {
        let mut cell = make_cell(1, 0, 0, 0);
        for (i, dir) in CardinalDirection::all().into_iter().enumerate() {
            cell.connect(dir, CellId(i as u32 + 2));
        }
        cell.disconnect(CardinalDirection::Nadir);
        let faces = cell.boundary_faces();
        assert_eq!(faces, vec![CardinalDirection::Nadir]);
    }
}
