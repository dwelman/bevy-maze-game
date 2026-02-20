use bevy::prelude::*;

use crate::{Controls, GameConfig};

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

/// Check if sweeping a collider of `size` from `start` to `end` would overlap
/// any other collider at any point along the path.
///
/// `self_entity` is excluded so an entity cannot collide with itself.
/// All other entities with a `Collider` are tested — including those that are
/// also moving — so turn-based entities block each other's intended destinations.
///
/// Samples N evenly-spaced positions along the path and performs a plain AABB
/// overlap test at each. N is chosen so the step between samples is always
/// smaller than the thinnest obstacle (WALL_T = 0.1) plus the mover's half-extent
/// in the movement direction, guaranteeing no obstacle is skipped.
fn check_swept_collision(
    self_entity: Entity,
    start: Vec3,
    end: Vec3,
    size: Vec3,
    collider_query: &Query<(Entity, &Transform, &Collider)>,
) -> bool {
    const MIN_OBSTACLE_THICKNESS: f32 = 0.1;

    let delta = end - start;
    let dist = delta.length();

    // Project the mover's half-extent onto the movement direction so we know
    // how much of the collider leads into a potential obstacle along that axis.
    let mover_half_extent = if dist > f32::EPSILON {
        let dir = delta / dist;
        (size * 0.5).dot(dir.abs())
    } else {
        0.0
    };

    // Step must be small enough that we cannot skip through the thinnest wall.
    let step_size = MIN_OBSTACLE_THICKNESS + mover_half_extent;
    let n = if dist > f32::EPSILON {
        ((dist / step_size).ceil() as usize).max(1) + 1
    } else {
        1
    };

    for i in 0..n {
        let t = if n == 1 { 0.0 } else { i as f32 / (n - 1) as f32 };
        let sample = start + delta * t;

        let mover_min = sample - size * 0.5;
        let mover_max = sample + size * 0.5;

        for (entity, other_transform, other_collider) in collider_query.iter() {
            if entity == self_entity {
                continue;
            }
            let obs_pos = other_transform.translation;
            let obs_min = obs_pos - other_collider.size * 0.5;
            let obs_max = obs_pos + other_collider.size * 0.5;

            let overlaps = mover_max.x > obs_min.x && mover_min.x < obs_max.x
                && mover_max.y > obs_min.y && mover_min.y < obs_max.y
                && mover_max.z > obs_min.z && mover_min.z < obs_max.z;

            if overlaps {
                debug!(
                    "Swept collision at t={:.2}: sample={:?} size={:?} obs_pos={:?} obs_size={:?}",
                    t, sample, size, obs_pos, other_collider.size
                );
                return true;
            }
        }
    }
    false
}

/// Snap a position to the nearest grid point based on grid_unit
fn snap_to_grid(position: Vec3, grid_unit: f32) -> Vec3 {
    Vec3::new(
        (position.x / grid_unit).round() * grid_unit,
        position.y, // Keep Y as-is (vertical position)
        (position.z / grid_unit).round() * grid_unit,
    )
}

pub fn update_lerp_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut queries: ParamSet<(
        Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
        Query<(Entity, &Transform, &Collider)>,
    )>,
) {
    let lerp_speed = config.player.lerp_speed;
    let grid_unit = config.player.grid_unit;
    let delta_time = time.delta_secs();

    // Collect all mover state up-front so we can borrow the collider query freely.
    let movers: Vec<(Entity, Vec3, Vec3, Vec3, MovementState, f32)> = queries
        .p0()
        .iter()
        .map(|(e, tf, mov, col)| {
            (e, tf.translation, col.size, mov.movement_delta, mov.state, mov.lerp_progress)
        })
        .collect();

    // For each mover, decide what to do, then apply mutations back via p0.
    for (entity, translation, col_size, movement_delta, state, lerp_progress) in movers {
        match state {
            MovementState::Idle => {
                if movement_delta == Vec3::ZERO {
                    continue;
                }

                let start = snap_to_grid(translation, grid_unit);
                let target = start + (movement_delta * grid_unit);

                // Read the forward direction from the current transform.
                let forward = {
                    let p0 = queries.p0();
                    let (_, tf, _, _) = p0.get(entity).unwrap();
                    tf.forward().as_vec3()
                };
                let right = forward.cross(Vec3::Y).normalize() * -1.0;

                // Swept collision check against ALL colliders (including other movers).
                let blocked = {
                    let collider_query = queries.p1();
                    check_swept_collision(entity, start, target, col_size, &collider_query)
                };

                if blocked {
                    debug!("Swept path blocked: {:?} -> {:?}", start, target);
                    queries.p0().get_mut(entity).unwrap().2.movement_delta = Vec3::ZERO;
                    continue;
                }

                // Diagonal corner-clip check.
                let movement_vec = movement_delta * grid_unit;
                let forward_component = movement_vec.dot(forward) * forward;
                let right_component = movement_vec.dot(right) * right;

                if forward_component.length_squared() > 0.01 && right_component.length_squared() > 0.01 {
                    let forward_target = start + forward_component;
                    let right_target = start + right_component;

                    let diag_blocked = {
                        let collider_query = queries.p1();
                        check_swept_collision(entity, start, forward_target, col_size, &collider_query)
                            || check_swept_collision(entity, start, right_target, col_size, &collider_query)
                    };

                    if diag_blocked {
                        debug!("Diagonal swept blocked");
                        queries.p0().get_mut(entity).unwrap().2.movement_delta = Vec3::ZERO;
                        continue;
                    }
                }

                // Commit movement.
                let mut query_p0 = queries.p0();
                let (_, tf, mut mov, _) = query_p0.get_mut(entity).unwrap();
                mov.start_position = start;
                mov.target_position = target;
                mov.lerp_progress = 0.0;
                mov.state = MovementState::MovingToTarget;
                mov.movement_delta = Vec3::ZERO;
                trace!(
                    "Move start: from={:?} to={:?} collider={:?}",
                    start, target, col_size
                );
                let _ = tf; // suppress unused warning; translation updated in MovingToTarget arm
            }
            MovementState::MovingToTarget => {
                let mut query_p0 = queries.p0();
                let (_, mut tf, mut mov, _) = query_p0.get_mut(entity).unwrap();
                let _ = lerp_progress; // we re-read from the component
                if mov.lerp_progress < 1.0 {
                    let next_progress = (mov.lerp_progress + lerp_speed * delta_time).min(1.0);
                    let next_position = mov.start_position.lerp(mov.target_position, next_progress);
                    trace!(
                        "Move step: progress={:.3} next={:?} target={:?}",
                        next_progress, next_position, mov.target_position
                    );
                    mov.lerp_progress = next_progress;
                    tf.translation = next_position;
                }
                if mov.lerp_progress >= 1.0 {
                    tf.translation = mov.target_position;
                    mov.state = MovementState::Idle;
                }
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

#[cfg(test)]
mod tests;