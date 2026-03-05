use bevy::prelude::*;

use crate::{Controls, GameConfig};
use crate::map::{CellGraph, CellId, CardinalDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Idle,
    MovingToTarget,
}

/// The cell an entity is moving towards. Set when movement begins, cleared on
/// arrival. Shared between systems (e.g. movement and visibility) so they
/// don't depend on each other directly.
#[derive(Component, Default)]
pub struct TargetCell(pub Option<CellId>);

/// The cardinal direction an entity is rotating towards. Set when rotation
/// begins, cleared on arrival. Mirrors [`TargetCell`] for facing changes.
#[derive(Component, Default)]
pub struct TargetFacing(pub Option<CardinalDirection>);

#[derive(Component)]
pub struct LerpMovement {
    /// The current state of movement
    pub state: MovementState,
    /// Requested movement direction (set by input, consumed by movement system)
    pub movement_delta: Vec3,
    /// The target position the player is moving towards
    pub target_position: Vec3,
    /// The starting position for the current lerp
    pub start_position: Vec3,
    /// Current lerp progress from 0.0 to 1.0
    pub lerp_progress: f32,
}

#[derive(Component)]
pub struct LerpRotation {
    /// Requested rotation delta in radians (set by input, consumed by rotation system)
    pub rotation_delta: f32,
    /// The target rotation the player is rotating towards
    pub target_rotation: Quat,
    /// Current lerp progress from 0.0 to 1.0
    pub lerp_progress: f32,
}

#[derive(Component)]
pub struct InputRepeatTimer {
    /// Tracks time since last movement input
    pub movement_timer: f32,
    /// Tracks time since last rotation input
    pub rotation_timer: f32,
}

/// Stores the entity's current position in the cell graph and its cardinal facing
/// direction. Both are kept up-to-date by the movement and rotation systems so that
/// neither needs to be re-derived from the `Transform` quaternion at runtime.
#[derive(Component)]
pub struct CellTransform {
    /// The cell the entity is currently occupying.
    pub cell: CellId,
    /// The cardinal direction the entity is facing (always N / S / E / W).
    pub facing: CardinalDirection,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Moves entities that have a [`CellTransform`] component by checking open
/// connections in the [`CellGraph`].
///
/// The stored `facing` is used to resolve the input delta into an absolute
/// cardinal [`CardinalDirection`] without touching the `Transform` quaternion.
/// Movement is silently blocked when no connection exists (i.e. there is a wall).
pub fn update_cell_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    cell_graph: Res<CellGraph>,
    mut query: Query<(&mut Transform, &mut LerpMovement, &mut CellTransform, &mut TargetCell)>,
) {
    let lerp_speed = config.player.lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut lerp_mov, mut cell_tf, mut target_cell) in &mut query {
        match lerp_mov.state {
            MovementState::Idle => {
                if lerp_mov.movement_delta == Vec3::ZERO {
                    continue;
                }

                let delta = lerp_mov.movement_delta;
                lerp_mov.movement_delta = Vec3::ZERO;

                // Resolve the world-space delta into a cardinal direction using
                // the stored facing — no quaternion math required.
                let fwd_vec = cell_tf.facing.to_vec3();
                let rgt_vec = cell_tf.facing.turn_right().to_vec3();
                let fwd_dot = delta.dot(fwd_vec);
                let rgt_dot = delta.dot(rgt_vec);

                let target_dir = if fwd_dot.abs() >= rgt_dot.abs() {
                    if fwd_dot >= 0.0 { cell_tf.facing } else { cell_tf.facing.opposite() }
                } else {
                    if rgt_dot >= 0.0 { cell_tf.facing.turn_right() } else { cell_tf.facing.turn_left() }
                };

                let Some(current_cell) = cell_graph.get_cell(cell_tf.cell) else {
                    continue;
                };

                let Some(neighbor_id) = current_cell.get_neighbor(target_dir) else {
                    debug!("Cell movement blocked: no {:?} connection from {:?}", target_dir, cell_tf.cell);
                    continue;
                };

                let Some(neighbor_cell) = cell_graph.get_cell(neighbor_id) else {
                    continue;
                };

                let start = transform.translation;
                let neighbor_pos = neighbor_cell.position();
                let target = Vec3::new(neighbor_pos.x, start.y, neighbor_pos.z);

                target_cell.0 = Some(neighbor_id);
                lerp_mov.start_position = start;
                lerp_mov.target_position = target;
                lerp_mov.lerp_progress = 0.0;
                lerp_mov.state = MovementState::MovingToTarget;
                debug!("Cell move: {:?} -> {:?} (dir={:?})", start, target, target_dir);
            }
            MovementState::MovingToTarget => {
                if lerp_mov.lerp_progress < 1.0 {
                    let next_progress =
                        (lerp_mov.lerp_progress + lerp_speed * delta_time).min(1.0);
                    lerp_mov.lerp_progress = next_progress;
                    transform.translation =
                        lerp_mov.start_position.lerp(lerp_mov.target_position, next_progress);
                }
                if lerp_mov.lerp_progress >= 1.0 {
                    transform.translation = lerp_mov.target_position;
                    if let Some(arrived_cell) = target_cell.0.take() {
                        cell_tf.cell = arrived_cell;
                    }
                    lerp_mov.state = MovementState::Idle;
                }
            }
        }
    }
}

pub fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    controls: Res<Controls>,
    mut player_query: Query<(
        &CellTransform,
        &mut LerpMovement,
        &mut LerpRotation,
        &mut InputRepeatTimer,
    )>,
    time: Res<Time>,
) {
    let repeat_delay = config.player.input_repeat_delay;
    let delta_time = time.delta_secs();

    for (cell_tf, mut player_mov, mut player_rot, mut input_timer) in &mut player_query {
        input_timer.movement_timer += delta_time;
        input_timer.rotation_timer += delta_time;

        // Only accept input when idle and not rotating
        if player_mov.state != MovementState::Idle || player_rot.lerp_progress < 1.0 {
            continue;
        }

        let mut movement_delta = Vec3::ZERO;
        let mut rotation_delta = 0.0;
        let mut movement_triggered = false;
        let mut rotation_triggered = false;

        let forward = cell_tf.facing.to_vec3();
        let right   = cell_tf.facing.turn_right().to_vec3();

        if keyboard_input.pressed(controls.move_forward) {
            if keyboard_input.just_pressed(controls.move_forward)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta += forward;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_backward) {
            if keyboard_input.just_pressed(controls.move_backward)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta -= forward;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_left) {
            if keyboard_input.just_pressed(controls.move_left)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta -= right;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_right) {
            if keyboard_input.just_pressed(controls.move_right)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta += right;
                movement_triggered = true;
            }
        }

        if keyboard_input.pressed(controls.rotate_left) {
            if keyboard_input.just_pressed(controls.rotate_left)
                || input_timer.rotation_timer >= repeat_delay
            {
                rotation_delta = std::f32::consts::FRAC_PI_2;
                rotation_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.rotate_right) {
            if keyboard_input.just_pressed(controls.rotate_right)
                || input_timer.rotation_timer >= repeat_delay
            {
                rotation_delta = -std::f32::consts::FRAC_PI_2;
                rotation_triggered = true;
            }
        }

        if movement_delta != Vec3::ZERO {
            player_mov.movement_delta = movement_delta;
            if movement_triggered {
                input_timer.movement_timer = 0.0;
            }
        }

        if rotation_delta != 0.0 {
            player_rot.rotation_delta = rotation_delta;
            if rotation_triggered {
                input_timer.rotation_timer = 0.0;
            }
        }
    }
}

pub fn update_lerp_rotation(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &mut LerpRotation, Option<&mut CellTransform>, Option<&mut TargetFacing>)>,
) {
    let rotation_lerp_speed = config.player.rotation_lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut rot, cell_tf, mut target_facing) in &mut query {
        if rot.rotation_delta != 0.0 && rot.lerp_progress >= 1.0 {
            // Compute and publish the target facing before the lerp begins.
            if let (Some(cell_tf), Some(target_facing)) = (&cell_tf, target_facing.as_mut()) {
                let new_facing = if rot.rotation_delta > 0.0 {
                    cell_tf.facing.turn_left()
                } else {
                    cell_tf.facing.turn_right()
                };
                target_facing.0 = Some(new_facing);
            }

            rot.target_rotation = transform.rotation * Quat::from_rotation_y(rot.rotation_delta);
            rot.lerp_progress = 0.0;
            rot.rotation_delta = 0.0;
        }

        if rot.lerp_progress < 1.0 {
            rot.lerp_progress = (rot.lerp_progress + rotation_lerp_speed * delta_time).min(1.0);
            let start_rotation = transform.rotation;
            transform.rotation = start_rotation.slerp(rot.target_rotation, rot.lerp_progress);
        } else {
            transform.rotation = rot.target_rotation;
        }

        // Keep CellTransform.facing in sync once the rotation settles.
        if rot.lerp_progress >= 1.0 {
            if let Some(mut cell_tf) = cell_tf {
                // Extract the world forward from the completed rotation.
                // Bevy objects face -Z by default; rotate that into world space.
                let world_fwd = transform.rotation * Vec3::NEG_Z;
                let new_facing = if world_fwd.x.abs() >= world_fwd.z.abs() {
                    if world_fwd.x >= 0.0 { CardinalDirection::East } else { CardinalDirection::West }
                } else {
                    if world_fwd.z >= 0.0 { CardinalDirection::South } else { CardinalDirection::North }
                };
                cell_tf.facing = new_facing;
            }
            if let Some(mut target_facing) = target_facing {
                target_facing.0 = None;
            }
        }
    }
}
