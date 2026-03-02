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
        if let Some(room) = self.rooms.get_mut(&room_id) {
            room.add_cell(cell_id);
            self.cell_to_room.insert(cell_id, room_id);
        }
    }

    pub fn get_cell_room(&self, cell_id: CellId) -> Option<RoomId> {
        self.cell_to_room.get(&cell_id).copied()
    }

    /// Records a cross-room connection in both rooms
    pub fn connect_rooms(
        &mut self,
        room_a: RoomId,
        cell_a: CellId,
        direction: Direction,
        room_b: RoomId,
        cell_b: CellId,
    ) {
        if let Some(room) = self.rooms.get_mut(&room_a) {
            room.add_exit(cell_a, direction, room_b, cell_b);
        }
        if let Some(room) = self.rooms.get_mut(&room_b) {
            room.add_exit(cell_b, direction.opposite(), room_a, cell_a);
        }
    }

    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }
}
