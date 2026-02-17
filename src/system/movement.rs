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

/// Check if placing a collider at the given position would result in a collision
fn check_collision(
    position: Vec3,
    size: Vec3,
    collider_query: &Query<(&Transform, &Collider), Without<LerpMovement>>,
) -> bool {
    for (other_transform, other_collider) in collider_query.iter() {
        if aabb_intersects(position, size, other_transform.translation, other_collider.size) {
            debug!(
                "Collision hit: pos={:?} size={:?} other_pos={:?} other_size={:?}",
                position,
                size,
                other_transform.translation,
                other_collider.size
            );
            return true;
        }
    }
    false
}

/// Check if two axis-aligned bounding boxes intersect
fn aabb_intersects(pos1: Vec3, size1: Vec3, pos2: Vec3, size2: Vec3) -> bool {
    let half_size1 = size1 * 0.5;
    let half_size2 = size2 * 0.5;
    
    let min1 = pos1 - half_size1;
    let max1 = pos1 + half_size1;
    let min2 = pos2 - half_size2;
    let max2 = pos2 + half_size2;
    
    max1.x > min2.x && min1.x < max2.x &&
    max1.y > min2.y && min1.y < max2.y &&
    max1.z > min2.z && min1.z < max2.z
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
    mut query: Query<(&mut Transform, &mut LerpMovement, &Collider)>,
    collider_query: Query<(&Transform, &Collider), Without<LerpMovement>>,
) {
    let lerp_speed = config.player.lerp_speed;
    let grid_unit = config.player.grid_unit;
    let delta_time = time.delta_secs();

    for (mut transform, mut mov, collider) in &mut query {
        
        match mov.state {
            MovementState::Idle => {
                // Check if there's a movement delta to process
                if mov.movement_delta != Vec3::ZERO {
                    // Initiate movement - apply grid_unit to the direction
                    // Snap start position to grid to prevent drift
                    mov.start_position = snap_to_grid(transform.translation, grid_unit);
                    mov.target_position = mov.start_position + (mov.movement_delta * grid_unit);
                    
                    // Check if target position would collide before starting movement
                    if check_collision(mov.target_position, collider.size, &collider_query) {
                        // Target is blocked - cancel movement immediately
                        debug!("Target position blocked: {:?}", mov.target_position);
                        mov.movement_delta = Vec3::ZERO; // Consume the delta
                        continue;
                    }
                    
                    // For diagonal movement, also check individual axis movements to prevent corner clipping
                    let forward = transform.forward().as_vec3();
                    let right = transform.right().as_vec3();
                    let movement_vec = mov.movement_delta * grid_unit;
                    
                    // Project movement onto forward and right axes
                    let forward_component = movement_vec.dot(forward) * forward;
                    let right_component = movement_vec.dot(right) * right;
                    
                    // If moving diagonally (both components non-zero), check each axis separately
                    if forward_component.length_squared() > 0.01 && right_component.length_squared() > 0.01 {
                        let forward_target = mov.start_position + forward_component;
                        let right_target = mov.start_position + right_component;
                        
                        if check_collision(forward_target, collider.size, &collider_query) {
                            debug!("Diagonal blocked by forward obstacle: {:?}", forward_target);
                            mov.movement_delta = Vec3::ZERO;
                            continue;
                        }
                        
                        if check_collision(right_target, collider.size, &collider_query) {
                            debug!("Diagonal blocked by side obstacle: {:?}", right_target);
                            mov.movement_delta = Vec3::ZERO;
                            continue;
                        }
                    }
                    
                    mov.lerp_progress = 0.0;
                    mov.state = MovementState::MovingToTarget;
                    mov.movement_delta = Vec3::ZERO; // Consume the delta
                    debug!(
                        "Move start: from={:?} to={:?} collider={:?}",
                        mov.start_position,
                        mov.target_position,
                        collider.size
                    );
                } else {
                    // Nothing to do, waiting for input
                    continue;
                }
            }
            MovementState::MovingToTarget => {
                if mov.lerp_progress < 1.0 {
                    // Continue lerping toward target (collision already validated before movement started)
                    let next_progress = (mov.lerp_progress + lerp_speed * delta_time).min(1.0);
                    let next_position = mov.start_position.lerp(mov.target_position, next_progress);

                    debug!(
                        "Move step: progress={:.3} next={:?} target={:?}",
                        next_progress,
                        next_position,
                        mov.target_position
                    );

                    mov.lerp_progress = next_progress;
                    transform.translation = next_position;
                }

                if mov.lerp_progress >= 1.0 {
                    // Movement complete
                    transform.translation = mov.target_position;
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