use bevy::prelude::*;
use std::collections::HashMap;
use crate::map::{CellGraph, Direction, RoomId, RoomMap};
use crate::GoalCellId;

const WALL_THICKNESS: f32 = 0.01;
const DOORWAY_WIDTH: f32 = 1.0;
const DOORWAY_HEIGHT: f32 = 2.0;

#[derive(Component)]
pub struct CellWall {
    pub cell_id: crate::map::CellId,
    pub direction: Direction,
}

#[derive(Component)]
pub struct CellDoorway {
    pub cell_id: crate::map::CellId,
    pub direction: Direction,
}

/// Spawns walls for all boundary faces in the cell graph
pub fn spawn_cell_walls(
    mut commands: Commands,
    graph: Res<CellGraph>,
    room_map: Res<RoomMap>,
    goal_cell: Res<GoalCellId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cell_size = graph.cell_size();

    // Pre-create a material for each room
    let room_materials: HashMap<RoomId, Handle<StandardMaterial>> = room_map.rooms()
        .map(|room| {
            let material = materials.add(StandardMaterial {
                base_color: room.color(),
                perceptual_roughness: 0.8,
                ..default()
            });
            (room.id(), material)
        })
        .collect();

    let default_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.8,
        ..default()
    });

    let goal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        perceptual_roughness: 0.8,
        ..default()
    });

    for cell in graph.cells() {
        let cell_position = cell.position();
        let cell_id = cell.id();

        let material = if goal_cell.0 == Some(cell_id) {
            goal_material.clone()
        } else if let Some(room_id) = room_map.get_cell_room(cell_id) {
            room_materials.get(&room_id).cloned().unwrap_or(default_material.clone())
        } else {
            default_material.clone()
        };

        // Spawn a wall for each boundary face
        for direction in cell.boundary_faces() {
            let (wall_size, wall_offset) = calculate_wall_geometry(direction, cell_size);
            let wall_position = cell_position + wall_offset;

            let mesh = meshes.add(Cuboid::new(wall_size.x, wall_size.y, wall_size.z));

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(wall_position),
                CellWall {
                    cell_id,
                    direction,
                },
            ));
        }

        // Spawn doorway frames at cross-room connections
        let cell_room = room_map.get_cell_room(cell_id);
        for direction in Direction::all() {
            if let Some(neighbor_id) = cell.get_neighbor(direction) {
                // Only spawn once per connection (from the lower-ID cell)
                if cell_id.0 >= neighbor_id.0 { continue; }

                let neighbor_room = room_map.get_cell_room(neighbor_id);

                // Only at boundaries between two different rooms
                if let (Some(room_a), Some(room_b)) = (cell_room, neighbor_room) {
                    if room_a != room_b {
                        let doorway_material = room_materials.get(&room_a)
                            .cloned()
                            .unwrap_or(default_material.clone());
                        for (piece_size, piece_offset) in calculate_doorway_pieces(direction, cell_size) {
                            let piece_position = cell_position + piece_offset;
                            let mesh = meshes.add(Cuboid::new(piece_size.x, piece_size.y, piece_size.z));
                            commands.spawn((
                                Mesh3d(mesh),
                                MeshMaterial3d(doorway_material.clone()),
                                Transform::from_translation(piece_position),
                                CellDoorway { cell_id, direction },
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Calculates the size and offset for a wall based on direction and cell size
fn calculate_wall_geometry(direction: Direction, cell_size: f32) -> (Vec3, Vec3) {
    let half_cell = cell_size / 2.0;

    match direction {
        Direction::North => {
            let size = Vec3::new(cell_size, cell_size, WALL_THICKNESS);
            let offset = Vec3::new(0.0, 0.0, -half_cell + WALL_THICKNESS / 2.0);
            (size, offset)
        }
        Direction::South => {
            let size = Vec3::new(cell_size, cell_size, WALL_THICKNESS);
            let offset = Vec3::new(0.0, 0.0, half_cell - WALL_THICKNESS / 2.0);
            (size, offset)
        }
        Direction::East => {
            let size = Vec3::new(WALL_THICKNESS, cell_size, cell_size);
            let offset = Vec3::new(half_cell - WALL_THICKNESS / 2.0, 0.0, 0.0);
            (size, offset)
        }
        Direction::West => {
            let size = Vec3::new(WALL_THICKNESS, cell_size, cell_size);
            let offset = Vec3::new(-half_cell + WALL_THICKNESS / 2.0, 0.0, 0.0);
            (size, offset)
        }
        Direction::Up => {
            let size = Vec3::new(cell_size, WALL_THICKNESS, cell_size);
            let offset = Vec3::new(0.0, half_cell - WALL_THICKNESS / 2.0, 0.0);
            (size, offset)
        }
        Direction::Down => {
            let size = Vec3::new(cell_size, WALL_THICKNESS, cell_size);
            let offset = Vec3::new(0.0, -half_cell + WALL_THICKNESS / 2.0, 0.0);
            (size, offset)
        }
    }
}

/// Calculates the geometry for the 3 frame pieces of a doorway (top bar, left side, right side).
/// The cutout is DOORWAY_WIDTH wide and DOORWAY_HEIGHT tall, centered horizontally, touching the floor.
/// Returns a Vec of (size, offset) pairs relative to the cell center.
fn calculate_doorway_pieces(direction: Direction, cell_size: f32) -> Vec<(Vec3, Vec3)> {
    let half_cell = cell_size / 2.0;
    let half_door_w = DOORWAY_WIDTH / 2.0;
    let door_top_y = -half_cell + DOORWAY_HEIGHT; // Y of top of cutout (relative to cell center)

    // Heights and vertical centers shared by all orientations
    let top_height = half_cell - door_top_y;         // cell_size - DOORWAY_HEIGHT
    let top_center_y = (door_top_y + half_cell) / 2.0;
    let side_width = half_cell - half_door_w;        // (cell_size - DOORWAY_WIDTH) / 2
    let side_center_y = (-half_cell + door_top_y) / 2.0;

    match direction {
        Direction::North => {
            let z = -half_cell + WALL_THICKNESS / 2.0;
            vec![
                // Top bar: full cell width
                (Vec3::new(cell_size, top_height, WALL_THICKNESS),
                 Vec3::new(0.0, top_center_y, z)),
                // Left side
                (Vec3::new(side_width, DOORWAY_HEIGHT, WALL_THICKNESS),
                 Vec3::new((-half_cell - half_door_w) / 2.0, side_center_y, z)),
                // Right side
                (Vec3::new(side_width, DOORWAY_HEIGHT, WALL_THICKNESS),
                 Vec3::new((half_door_w + half_cell) / 2.0, side_center_y, z)),
            ]
        }
        Direction::South => {
            let z = half_cell - WALL_THICKNESS / 2.0;
            vec![
                (Vec3::new(cell_size, top_height, WALL_THICKNESS),
                 Vec3::new(0.0, top_center_y, z)),
                (Vec3::new(side_width, DOORWAY_HEIGHT, WALL_THICKNESS),
                 Vec3::new((-half_cell - half_door_w) / 2.0, side_center_y, z)),
                (Vec3::new(side_width, DOORWAY_HEIGHT, WALL_THICKNESS),
                 Vec3::new((half_door_w + half_cell) / 2.0, side_center_y, z)),
            ]
        }
        Direction::East => {
            let x = half_cell - WALL_THICKNESS / 2.0;
            vec![
                // Top bar: full cell width along Z
                (Vec3::new(WALL_THICKNESS, top_height, cell_size),
                 Vec3::new(x, top_center_y, 0.0)),
                // Left side (along -Z)
                (Vec3::new(WALL_THICKNESS, DOORWAY_HEIGHT, side_width),
                 Vec3::new(x, side_center_y, (-half_cell - half_door_w) / 2.0)),
                // Right side (along +Z)
                (Vec3::new(WALL_THICKNESS, DOORWAY_HEIGHT, side_width),
                 Vec3::new(x, side_center_y, (half_door_w + half_cell) / 2.0)),
            ]
        }
        Direction::West => {
            let x = -half_cell + WALL_THICKNESS / 2.0;
            vec![
                (Vec3::new(WALL_THICKNESS, top_height, cell_size),
                 Vec3::new(x, top_center_y, 0.0)),
                (Vec3::new(WALL_THICKNESS, DOORWAY_HEIGHT, side_width),
                 Vec3::new(x, side_center_y, (-half_cell - half_door_w) / 2.0)),
                (Vec3::new(WALL_THICKNESS, DOORWAY_HEIGHT, side_width),
                 Vec3::new(x, side_center_y, (half_door_w + half_cell) / 2.0)),
            ]
        }
        Direction::Up | Direction::Down => vec![],
    }
}
