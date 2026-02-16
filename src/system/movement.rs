use bevy::prelude::*;

use crate::{Controls, GameConfig};

#[derive(Component)]
pub struct LerpMovement {
    /// The target position the player is moving towards
    pub target_position: Vec3,
    /// Current lerp progress from 0.0 to 1.0
    pub lerp_progress: f32,
}

#[derive(Component)]
pub struct LerpRotation {
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
    let grid_unit = config.player.grid_unit;
    let repeat_delay = config.player.input_repeat_delay;
    let delta_time = time.delta_secs();

    for (transform, mut player_mov, mut player_rot, mut input_timer) in &mut player_query {
        // Update timers
        input_timer.movement_timer += delta_time;
        input_timer.rotation_timer += delta_time;

        let mut movement_delta = Vec3::ZERO;
        let mut rotation_delta = 0.0;
        let mut movement_triggered = false;
        let mut rotation_triggered = false;

        // Calculate movement relative to player's facing direction
        let forward = transform.forward();
        let right = transform.right();

        // Check movement inputs
        if keyboard_input.pressed(controls.move_forward) {
            if keyboard_input.just_pressed(controls.move_forward)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta += forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_backward) {
            if keyboard_input.just_pressed(controls.move_backward)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta -= forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_left) {
            if keyboard_input.just_pressed(controls.move_left)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta -= right * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_right) {
            if keyboard_input.just_pressed(controls.move_right)
                || input_timer.movement_timer >= repeat_delay
            {
                movement_delta += right * grid_unit;
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

        if movement_delta != Vec3::ZERO {
            player_mov.target_position += movement_delta;
            player_mov.lerp_progress = 0.0;
            if movement_triggered {
                input_timer.movement_timer = 0.0;
            }
        }
        if rotation_delta != 0.0 {
            let current_rotation = player_rot.target_rotation;
            player_rot.target_rotation = current_rotation * Quat::from_rotation_y(rotation_delta);
            player_rot.lerp_progress = 0.0;
            if rotation_triggered {
                input_timer.rotation_timer = 0.0;
            }
        }
    }
}

pub fn update_lerp_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &mut LerpMovement)>,
) {
    let lerp_speed = config.player.lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut mov) in &mut query {
        if mov.lerp_progress < 1.0 {
            // Increase lerp progress
            mov.lerp_progress = (mov.lerp_progress + lerp_speed * delta_time).min(1.0);

            // Lerp position (start from current position towards target)
            let current_pos = transform.translation;
            transform.translation = current_pos.lerp(mov.target_position, mov.lerp_progress);
        } else {
            // Ensure we're at the exact target position
            transform.translation = mov.target_position;
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
