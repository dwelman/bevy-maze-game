use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::map::{CellId, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomId(pub u32);

#[derive(Debug)]
pub struct Room {
    id: RoomId,
    cells: HashSet<CellId>,
    color: Color,
    /// Maps (cell_id, direction) to (target_room_id, target_cell_id) for cross-room connections
    exits: HashMap<(CellId, Direction), (RoomId, CellId)>,
}

impl Room {
    pub fn new(id: RoomId, color: Color) -> Self {
        Self {
            id,
            cells: HashSet::new(),
            color,
            exits: HashMap::new(),
        }
    }

    pub fn id(&self) -> RoomId {
        self.id
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn add_cell(&mut self, cell_id: CellId) {
        self.cells.insert(cell_id);
    }

    pub fn remove_cell(&mut self, cell_id: CellId) {
        self.cells.remove(&cell_id);
    }

    pub fn cells(&self) -> impl Iterator<Item = CellId> + '_ {
        self.cells.iter().copied()
    }

    pub fn contains_cell(&self, cell_id: CellId) -> bool {
        self.cells.contains(&cell_id)
    }

    pub fn add_exit(&mut self, cell_id: CellId, direction: Direction, target_room: RoomId, target_cell: CellId) {
        self.exits.insert((cell_id, direction), (target_room, target_cell));
    }

    pub fn exits(&self) -> impl Iterator<Item = (&(CellId, Direction), &(RoomId, CellId))> {
        self.exits.iter()
    }
}

#[derive(Resource)]
pub struct RoomMap {
    rooms: HashMap<RoomId, Room>,
    cell_to_room: HashMap<CellId, RoomId>,
    next_room_id: u32,
}

impl RoomMap {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            cell_to_room: HashMap::new(),
            next_room_id: 0,
        }
    }

    pub fn create_room(&mut self, color: Color) -> RoomId {
        let id = RoomId(self.next_room_id);
        self.next_room_id += 1;
        self.rooms.insert(id, Room::new(id, color));
        id
    }

    pub fn get_room(&self, room_id: RoomId) -> Option<&Room> {
        self.rooms.get(&room_id)
    }

    pub fn get_room_mut(&mut self, room_id: RoomId) -> Option<&mut Room> {
        self.rooms.get_mut(&room_id)
    }

    pub fn assign_cell(&mut self, cell_id: CellId, room_id: RoomId) {
        assert!(self.rooms.contains_key(&room_id), "assign_cell: room {:?} does not exist", room_id);

        // Remove the cell from its current room before reassigning.
        if let Some(&old_room_id) = self.cell_to_room.get(&cell_id) {
            if old_room_id != room_id {
                if let Some(old_room) = self.rooms.get_mut(&old_room_id) {
                    old_room.remove_cell(cell_id);
                }
            }
        }

        self.rooms.get_mut(&room_id).unwrap().add_cell(cell_id);
        self.cell_to_room.insert(cell_id, room_id);
    }

    pub fn get_cell_room(&self, cell_id: CellId) -> Option<RoomId> {
        self.cell_to_room.get(&cell_id).copied()
    }

    /// Records a cross-room connection in both rooms.
    ///
    /// Panics if either room does not exist.
    /// In debug builds, also panics if `cell_a` does not belong to `room_a`
    /// or `cell_b` does not belong to `room_b`.
    pub fn connect_rooms(
        &mut self,
        room_a: RoomId,
        cell_a: CellId,
        direction: Direction,
        room_b: RoomId,
        cell_b: CellId,
    ) {
        assert!(self.rooms.contains_key(&room_a), "connect_rooms: room_a {:?} does not exist", room_a);
        assert!(self.rooms.contains_key(&room_b), "connect_rooms: room_b {:?} does not exist", room_b);
        debug_assert!(
            self.rooms[&room_a].contains_cell(cell_a),
            "connect_rooms: cell {:?} does not belong to room_a {:?}",
            cell_a, room_a,
        );
        debug_assert!(
            self.rooms[&room_b].contains_cell(cell_b),
            "connect_rooms: cell {:?} does not belong to room_b {:?}",
            cell_b, room_b,
        );

        self.rooms.get_mut(&room_a).unwrap().add_exit(cell_a, direction, room_b, cell_b);
        self.rooms.get_mut(&room_b).unwrap().add_exit(cell_b, direction.opposite(), room_a, cell_a);
    }

    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers

    fn red() -> Color { Color::srgb(1.0, 0.0, 0.0) }
    fn blue() -> Color { Color::srgb(0.0, 0.0, 1.0) }

    fn make_room(id: u32, color: Color) -> Room {
        Room::new(RoomId(id), color)
    }

    fn sorted_cells(room: &Room) -> Vec<u32> {
        let mut v: Vec<u32> = room.cells().map(|c| c.0).collect();
        v.sort();
        v
    }

    // -------------------------------------------------------------------------
    // Room::new
    // -------------------------------------------------------------------------

    #[test]
    fn room_new_stores_id_and_color() {
        let room = make_room(7, red());
        assert_eq!(room.id(), RoomId(7));
        assert_eq!(room.color(), red());
    }

    #[test]
    fn room_new_starts_empty() {
        let room = make_room(1, red());
        assert_eq!(sorted_cells(&room), Vec::<u32>::new());
        assert_eq!(room.exits().count(), 0);
    }

    #[test]
    fn room_new_id_zero() {
        let room = make_room(0, red());
        assert_eq!(room.id(), RoomId(0));
    }

    #[test]
    fn room_new_id_max_u32() {
        let room = make_room(u32::MAX, red());
        assert_eq!(room.id(), RoomId(u32::MAX));
    }

    // -------------------------------------------------------------------------
    // Room::color
    // -------------------------------------------------------------------------

    #[test]
    fn room_color_returns_construction_value() {
        let room = make_room(1, blue());
        assert_eq!(room.color(), blue());
    }

    // -------------------------------------------------------------------------
    // Room::add_cell / contains_cell
    // -------------------------------------------------------------------------

    #[test]
    fn add_cell_makes_contains_cell_true() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(10));
        assert!(room.contains_cell(CellId(10)));
    }

    #[test]
    fn contains_cell_false_when_not_added() {
        let room = make_room(1, red());
        assert!(!room.contains_cell(CellId(42)));
    }

    #[test]
    fn add_cell_twice_is_idempotent() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(5));
        room.add_cell(CellId(5));
        assert_eq!(sorted_cells(&room), vec![5]);
    }

    #[test]
    fn add_multiple_cells() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(1));
        room.add_cell(CellId(2));
        room.add_cell(CellId(3));
        assert_eq!(sorted_cells(&room), vec![1, 2, 3]);
    }

    // -------------------------------------------------------------------------
    // Room::remove_cell
    // -------------------------------------------------------------------------

    #[test]
    fn remove_cell_makes_contains_cell_false() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(10));
        room.remove_cell(CellId(10));
        assert!(!room.contains_cell(CellId(10)));
    }

    #[test]
    fn remove_cell_not_present_is_noop() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(1));
        room.remove_cell(CellId(99)); // not present
        assert_eq!(sorted_cells(&room), vec![1]);
    }

    #[test]
    fn remove_cell_only_removes_target() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(1));
        room.add_cell(CellId(2));
        room.remove_cell(CellId(1));
        assert!(!room.contains_cell(CellId(1)));
        assert!(room.contains_cell(CellId(2)));
    }

    // -------------------------------------------------------------------------
    // Room::cells
    // -------------------------------------------------------------------------

    #[test]
    fn cells_iterator_empty_on_new_room() {
        let room = make_room(1, red());
        assert_eq!(room.cells().count(), 0);
    }

    #[test]
    fn cells_iterator_contains_all_added_cells() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(10));
        room.add_cell(CellId(20));
        room.add_cell(CellId(30));
        assert_eq!(sorted_cells(&room), vec![10, 20, 30]);
    }

    #[test]
    fn cells_iterator_reflects_removals() {
        let mut room = make_room(1, red());
        room.add_cell(CellId(10));
        room.add_cell(CellId(20));
        room.remove_cell(CellId(10));
        assert_eq!(sorted_cells(&room), vec![20]);
    }

    // -------------------------------------------------------------------------
    // Room::add_exit / exits
    // -------------------------------------------------------------------------

    #[test]
    fn add_exit_appears_in_exits_iterator() {
        let mut room = make_room(1, red());
        room.add_exit(CellId(1), Direction::North, RoomId(2), CellId(5));
        let exits: Vec<_> = room.exits().collect();
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].0, &(CellId(1), Direction::North));
        assert_eq!(exits[0].1, &(RoomId(2), CellId(5)));
    }

    #[test]
    fn exits_empty_on_new_room() {
        let room = make_room(1, red());
        assert_eq!(room.exits().count(), 0);
    }

    #[test]
    fn add_exit_multiple_exits() {
        let mut room = make_room(1, red());
        room.add_exit(CellId(1), Direction::North, RoomId(2), CellId(10));
        room.add_exit(CellId(1), Direction::East, RoomId(3), CellId(11));
        assert_eq!(room.exits().count(), 2);
    }

    #[test]
    fn add_exit_overwrites_same_key() {
        let mut room = make_room(1, red());
        room.add_exit(CellId(1), Direction::North, RoomId(2), CellId(10));
        room.add_exit(CellId(1), Direction::North, RoomId(9), CellId(99));
        let exits: Vec<_> = room.exits().collect();
        assert_eq!(exits.len(), 1);
        assert_eq!(exits[0].1, &(RoomId(9), CellId(99)));
    }

    // -------------------------------------------------------------------------
    // RoomMap::new
    // -------------------------------------------------------------------------

    #[test]
    fn roommap_new_has_no_rooms() {
        let map = RoomMap::new();
        assert_eq!(map.rooms().count(), 0);
    }

    // -------------------------------------------------------------------------
    // RoomMap::create_room
    // -------------------------------------------------------------------------

    #[test]
    fn create_room_returns_accessible_room() {
        let mut map = RoomMap::new();
        let id = map.create_room(red());
        assert!(map.get_room(id).is_some());
    }

    #[test]
    fn create_room_ids_are_sequential() {
        let mut map = RoomMap::new();
        let id0 = map.create_room(red());
        let id1 = map.create_room(blue());
        let id2 = map.create_room(red());
        assert_eq!(id0, RoomId(0));
        assert_eq!(id1, RoomId(1));
        assert_eq!(id2, RoomId(2));
    }

    #[test]
    fn create_room_stores_correct_color() {
        let mut map = RoomMap::new();
        let id = map.create_room(blue());
        assert_eq!(map.get_room(id).unwrap().color(), blue());
    }

    #[test]
    fn create_room_increments_total_count() {
        let mut map = RoomMap::new();
        map.create_room(red());
        map.create_room(blue());
        assert_eq!(map.rooms().count(), 2);
    }

    // -------------------------------------------------------------------------
    // RoomMap::get_room / get_room_mut
    // -------------------------------------------------------------------------

    #[test]
    fn get_room_returns_none_for_missing_id() {
        let map = RoomMap::new();
        assert!(map.get_room(RoomId(99)).is_none());
    }

    #[test]
    fn get_room_mut_allows_mutation() {
        let mut map = RoomMap::new();
        let id = map.create_room(red());
        map.get_room_mut(id).unwrap().add_cell(CellId(42));
        assert!(map.get_room(id).unwrap().contains_cell(CellId(42)));
    }

    #[test]
    fn get_room_mut_returns_none_for_missing_id() {
        let mut map = RoomMap::new();
        assert!(map.get_room_mut(RoomId(0)).is_none());
    }

    // -------------------------------------------------------------------------
    // RoomMap::assign_cell / get_cell_room
    // -------------------------------------------------------------------------

    #[test]
    fn assign_cell_registers_cell_in_room() {
        let mut map = RoomMap::new();
        let rid = map.create_room(red());
        map.assign_cell(CellId(1), rid);
        assert!(map.get_room(rid).unwrap().contains_cell(CellId(1)));
        assert_eq!(map.get_cell_room(CellId(1)), Some(rid));
    }

    #[test]
    fn get_cell_room_returns_none_for_unassigned_cell() {
        let map = RoomMap::new();
        assert_eq!(map.get_cell_room(CellId(99)), None);
    }

    #[test]
    fn assign_cell_to_same_room_twice_is_idempotent() {
        let mut map = RoomMap::new();
        let rid = map.create_room(red());
        map.assign_cell(CellId(1), rid);
        map.assign_cell(CellId(1), rid);
        let cells: Vec<_> = map.get_room(rid).unwrap().cells().collect();
        assert_eq!(cells.len(), 1);
        assert_eq!(map.get_cell_room(CellId(1)), Some(rid));
    }

    #[test]
    fn assign_cell_moves_cell_between_rooms() {
        let mut map = RoomMap::new();
        let r1 = map.create_room(red());
        let r2 = map.create_room(blue());
        map.assign_cell(CellId(1), r1);
        map.assign_cell(CellId(1), r2);

        assert!(!map.get_room(r1).unwrap().contains_cell(CellId(1)));
        assert!(map.get_room(r2).unwrap().contains_cell(CellId(1)));
        assert_eq!(map.get_cell_room(CellId(1)), Some(r2));
    }

    #[test]
    #[should_panic]
    fn assign_cell_panics_for_nonexistent_room() {
        let mut map = RoomMap::new();
        map.assign_cell(CellId(1), RoomId(99));
    }

    // -------------------------------------------------------------------------
    // RoomMap::connect_rooms
    // -------------------------------------------------------------------------

    #[test]
    fn connect_rooms_adds_exit_in_both_rooms() {
        let mut map = RoomMap::new();
        let ra = map.create_room(red());
        let rb = map.create_room(blue());
        map.assign_cell(CellId(1), ra);
        map.assign_cell(CellId(2), rb);

        map.connect_rooms(ra, CellId(1), Direction::North, rb, CellId(2));

        let room_a = map.get_room(ra).unwrap();
        let room_b = map.get_room(rb).unwrap();

        assert!(room_a.exits().any(|(k, v)| *k == (CellId(1), Direction::North) && *v == (rb, CellId(2))));
        assert!(room_b.exits().any(|(k, v)| *k == (CellId(2), Direction::South) && *v == (ra, CellId(1))));
    }

    #[test]
    fn connect_rooms_uses_opposite_direction_for_return_exit() {
        let mut map = RoomMap::new();
        let ra = map.create_room(red());
        let rb = map.create_room(blue());
        map.assign_cell(CellId(1), ra);
        map.assign_cell(CellId(2), rb);

        map.connect_rooms(ra, CellId(1), Direction::East, rb, CellId(2));

        let room_b = map.get_room(rb).unwrap();
        assert!(room_b.exits().any(|(k, _)| k.1 == Direction::West));
    }

    #[test]
    #[should_panic]
    fn connect_rooms_panics_if_room_a_missing() {
        let mut map = RoomMap::new();
        let rb = map.create_room(blue());
        map.assign_cell(CellId(2), rb);
        map.connect_rooms(RoomId(99), CellId(1), Direction::North, rb, CellId(2));
    }

    #[test]
    #[should_panic]
    fn connect_rooms_panics_if_room_b_missing() {
        let mut map = RoomMap::new();
        let ra = map.create_room(red());
        map.assign_cell(CellId(1), ra);
        map.connect_rooms(ra, CellId(1), Direction::North, RoomId(99), CellId(2));
    }

    // -------------------------------------------------------------------------
    // RoomMap::rooms
    // -------------------------------------------------------------------------

    #[test]
    fn rooms_iterator_empty_on_new_map() {
        let map = RoomMap::new();
        assert_eq!(map.rooms().count(), 0);
    }

    #[test]
    fn rooms_iterator_covers_all_created_rooms() {
        let mut map = RoomMap::new();
        let r1 = map.create_room(red());
        let r2 = map.create_room(blue());
        let ids: Vec<RoomId> = map.rooms().map(|r| r.id()).collect();
        assert!(ids.contains(&r1));
        assert!(ids.contains(&r2));
        assert_eq!(ids.len(), 2);
    }
}
