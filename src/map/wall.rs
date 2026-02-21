use bevy::prelude::*;
use crate::map::{CellGraph, Direction};
use crate::system::movement::Collider;

const WALL_THICKNESS: f32 = 0.1;

#[derive(Component)]
pub struct CellWall {
    pub cell_id: crate::map::CellId,
    pub direction: Direction,
}

/// Spawns walls for all boundary faces in the cell graph
pub fn spawn_cell_walls(
    mut commands: Commands,
    graph: Res<CellGraph>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cell_size = graph.cell_size();

    // Material for walls
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.8,
        ..default()
    });

    for cell in graph.cells() {
        let cell_position = cell.position();
        let cell_id = cell.id();

        // Spawn a wall for each boundary face
        for direction in cell.boundary_faces() {
            let (wall_size, wall_offset) = calculate_wall_geometry(direction, cell_size);
            let wall_position = cell_position + wall_offset;

            // Create mesh for this wall
            let mesh = meshes.add(Cuboid::new(wall_size.x, wall_size.y, wall_size.z));

            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(wall_material.clone()),
                Transform::from_translation(wall_position),
                Collider { size: wall_size },
                CellWall {
                    cell_id,
                    direction,
                },
            ));
        }
    }
}

/// Calculates the size and offset for a wall based on direction and cell size
fn calculate_wall_geometry(direction: Direction, cell_size: f32) -> (Vec3, Vec3) {
    let half_cell = cell_size / 2.0;

    match direction {
        Direction::North => {
            // Wall on +Z face
            let size = Vec3::new(cell_size, cell_size, WALL_THICKNESS);
            let offset = Vec3::new(0.0, 0.0, half_cell);
            (size, offset)
        }
        Direction::South => {
            // Wall on -Z face
            let size = Vec3::new(cell_size, cell_size, WALL_THICKNESS);
            let offset = Vec3::new(0.0, 0.0, -half_cell);
            (size, offset)
        }
        Direction::East => {
            // Wall on +X face
            let size = Vec3::new(WALL_THICKNESS, cell_size, cell_size);
            let offset = Vec3::new(half_cell, 0.0, 0.0);
            (size, offset)
        }
        Direction::West => {
            // Wall on -X face
            let size = Vec3::new(WALL_THICKNESS, cell_size, cell_size);
            let offset = Vec3::new(-half_cell, 0.0, 0.0);
            (size, offset)
        }
        Direction::Up => {
            // Wall on +Y face (ceiling)
            let size = Vec3::new(cell_size, WALL_THICKNESS, cell_size);
            let offset = Vec3::new(0.0, half_cell, 0.0);
            (size, offset)
        }
        Direction::Down => {
            // Wall on -Y face (floor)
            let size = Vec3::new(cell_size, WALL_THICKNESS, cell_size);
            let offset = Vec3::new(0.0, -half_cell, 0.0);
            (size, offset)
        }
    }
}
