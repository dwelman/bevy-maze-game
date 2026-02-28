use bevy::prelude::*;

use crate::{Controls, GameConfig};
use crate::map::{CellGraph, CellId, Direction};

#[derive(Component)]
pub struct Collider {
    pub size: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Idle,
    MovingToTarget,
}

#[derive(Component)]
pub struct LerpMovement {
    /// The current state of movement
    pub state: MovementState,
    /// Requested movement delta (set by input, consumed by movement system)
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

/// Cell-based movement component. Entities with this component move by checking
/// open connections between cells rather than using swept collision detection.
#[derive(Component)]
pub struct CellMovement {
    /// The cell the entity is currently standing in.
    pub current_cell: CellId,
}

/// Maps a world-space movement vector to the most appropriate horizontal cardinal
/// `Direction`, ignoring the Y component. Returns `None` for zero vectors.
fn world_vec_to_direction(v: Vec3) -> Option<Direction> {
    let x = v.x;
    let z = v.z;
    if x.abs() < f32::EPSILON && z.abs() < f32::EPSILON {
        return None;
    }
    // Dominant axis wins; ties go to X (East/West)
    if x.abs() >= z.abs() {
        if x > 0.0 { Some(Direction::East) } else { Some(Direction::West) }
    } else {
        if z > 0.0 { Some(Direction::North) } else { Some(Direction::South) }
    }
}

/// Moves entities that have a [`CellMovement`] component by checking open connections
/// in the [`CellGraph`] rather than using swept collision detection.
///
/// When idle with a pending `movement_delta`, the system converts the world-space
/// direction to a cardinal [`Direction`], verifies there is a connected neighbour cell,
/// and initiates a lerp to that neighbour's centre position. Movement is blocked
/// silently when no connection exists (i.e. there is a wall).
pub fn update_cell_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    cell_graph: Res<CellGraph>,
    mut query: Query<(&mut Transform, &mut LerpMovement, &mut CellMovement)>,
) {
    let lerp_speed = config.player.lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut lerp_mov, mut cell_mov) in &mut query {
        match lerp_mov.state {
            MovementState::Idle => {
                if lerp_mov.movement_delta == Vec3::ZERO {
                    continue;
                }

                let delta = lerp_mov.movement_delta;
                lerp_mov.movement_delta = Vec3::ZERO;

                let Some(move_dir) = world_vec_to_direction(delta) else {
                    continue;
                };

                let Some(current_cell) = cell_graph.get_cell(cell_mov.current_cell) else {
                    continue;
                };

                let Some(neighbor_id) = current_cell.get_neighbor(move_dir) else {
                    debug!("Cell movement blocked: no {:?} connection from {:?}", move_dir, cell_mov.current_cell);
                    continue;
                };

                let Some(neighbor_cell) = cell_graph.get_cell(neighbor_id) else {
                    continue;
                };

                let start = transform.translation;
                let neighbor_pos = neighbor_cell.position();
                // Preserve the entity's Y so it stays at the correct floor height
                let target = Vec3::new(neighbor_pos.x, start.y, neighbor_pos.z);

                cell_mov.current_cell = neighbor_id;
                lerp_mov.start_position = start;
                lerp_mov.target_position = target;
                lerp_mov.lerp_progress = 0.0;
                lerp_mov.state = MovementState::MovingToTarget;
                debug!("Cell move: {:?} -> {:?} (dir={:?})", start, target, move_dir);
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
        &Transform,
        &mut LerpMovement,
        &mut LerpRotation,
        &mut InputRepeatTimer,
    )>,
    time: Res<Time>,
) {
    let repeat_delay = config.player.input_repeat_delay;
    let delta_time = time.delta_secs();

    for (transform, mut player_mov, mut player_rot, mut input_timer) in &mut player_query {
        // Update timers
        input_timer.movement_timer += delta_time;
        input_timer.rotation_timer += delta_time;

        // Only accept input when idle (not currently moving)
        if player_mov.state != MovementState::Idle {
            continue;
        }

        let mut movement_delta = Vec3::ZERO;
        let mut rotation_delta = 0.0;
        let mut movement_triggered = false;
        let mut rotation_triggered = false;

        // Calculate movement direction relative to player's facing direction
        let forward = transform.forward().as_vec3();
        let right = transform.right().as_vec3();

        // Check movement inputs - just set direction, movement system handles distance
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

        // Check rotation inputs
        if keyboard_input.pressed(controls.rotate_left) {
            if keyboard_input.just_pressed(controls.rotate_left)
                || input_timer.rotation_timer >= repeat_delay
            {
                rotation_delta = std::f32::consts::FRAC_PI_2; // 90 degrees
                rotation_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.rotate_right) {
            if keyboard_input.just_pressed(controls.rotate_right)
                || input_timer.rotation_timer >= repeat_delay
            {
                rotation_delta = -std::f32::consts::FRAC_PI_2; // -90 degrees
                rotation_triggered = true;
            }
        }

        // Set movement delta - the movement system will handle the rest
        if movement_delta != Vec3::ZERO {
            player_mov.movement_delta = movement_delta;
            if movement_triggered {
                input_timer.movement_timer = 0.0;
            }
        }
        
        // Set rotation delta - the rotation system will handle the rest
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
    mut query: Query<(&mut Transform, &mut LerpRotation)>,
) {
    let rotation_lerp_speed = config.player.rotation_lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut rot) in &mut query {
        // Check if there's a rotation delta to process
        if rot.rotation_delta != 0.0 && rot.lerp_progress >= 1.0 {
            // Initiate rotation
            rot.target_rotation = transform.rotation * Quat::from_rotation_y(rot.rotation_delta);
            rot.lerp_progress = 0.0;
            rot.rotation_delta = 0.0; // Consume the delta
        }
        
        if rot.lerp_progress < 1.0 {
            // Increase lerp progress
            rot.lerp_progress = (rot.lerp_progress + rotation_lerp_speed * delta_time).min(1.0);

            // Slerp rotation towards target
            let start_rotation = transform.rotation;
            transform.rotation = start_rotation.slerp(rot.target_rotation, rot.lerp_progress);
        } else {
            // Ensure we're at the exact target rotation
            transform.rotation = rot.target_rotation;
        }
    }
}
