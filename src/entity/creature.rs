use bevy::prelude::*;

use crate::system::movement::{Collider, InputRepeatTimer, LerpMovement, LerpRotation, MovementState};

/// Defines the physical shape of a creature.
///
/// Dimensions are in world units. With `grid_unit = 1.0` a creature with
/// `height = 1.75` occupies cells at `y = floor_coord` and `y = floor_coord + 1`
/// (since 1.75 > 1.0 it spills into the cell above).
#[derive(Clone, Debug)]
pub struct CreatureDef {
    /// Total height in world units.
    pub height: f32,
    /// Width (and depth) of the creature's collider in world units.
    pub width: f32,
    /// Height above the creature's feet where the eyes (camera) sit.
    pub eye_height: f32,
}

/// Marker component attached to every creature entity.
#[derive(Component)]
pub struct Creature;

/// Spawns a creature into the world, returning its `Entity`.
///
/// `grid_coord` is the **bottom cell** the creature occupies. Its feet are
/// placed at the floor of that cell, i.e. `y = grid_coord.y * grid_unit`.
/// The entity `Transform` is centred at `y = feet + height / 2` so the
/// `Collider` (also centred on the transform) spans the correct vertical range.
///
/// The returned entity has [`Creature`], [`Collider`], [`LerpMovement`],
/// [`LerpRotation`], and [`InputRepeatTimer`] but **no camera** — attach a
/// camera child with [`with_children`](bevy::hierarchy::BuildChildren::with_children)
/// on the returned entity if needed.
pub fn spawn_creature(
    commands: &mut Commands,
    grid_coord: IVec3,
    grid_unit: f32,
    def: &CreatureDef,
) -> Entity {
    // Feet sit at the floor of the bottom cell.
    let feet_y = grid_coord.y as f32 * grid_unit;
    // Entity origin is the vertical centre of the collider.
    let origin = Vec3::new(
        grid_coord.x as f32 * grid_unit,
        feet_y + def.height * 0.5,
        grid_coord.z as f32 * grid_unit,
    );

    commands.spawn((
        Creature,
        Collider {
            size: Vec3::new(def.width, def.height, def.width),
        },
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: origin,
            start_position: origin,
            lerp_progress: 1.0,
        },
        LerpRotation {
            rotation_delta: 0.0,
            target_rotation: Quat::IDENTITY,
            lerp_progress: 1.0,
        },
        InputRepeatTimer {
            movement_timer: 0.0,
            rotation_timer: 0.0,
        },
        Transform::from_translation(origin),
    )).id()
}
