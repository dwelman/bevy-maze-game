use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{CameraLookMode, Controls, GameConfig, LookMode};

#[derive(Component)]
pub struct CameraLook {
    /// Current horizontal look angle in radians (positive = right, negative = left)
    pub yaw: f32,
    /// Current vertical look angle in radians (positive = up, negative = down)
    pub pitch: f32,
    /// Target horizontal look angle we're lerping towards
    pub target_yaw: f32,
    /// Target vertical look angle we're lerping towards
    pub target_pitch: f32,
}

pub fn handle_camera_look(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    controls: Res<Controls>,
    look_mode: Res<CameraLookMode>,
    mouse_motion: MessageReader<MouseMotion>,
    camera_query: Query<(&mut Transform, &mut CameraLook), With<Camera3d>>,
    cursor_query: Query<&Window, With<PrimaryWindow>>,
) {
    match &look_mode.0 {
        LookMode::Relative => {
            handle_camera_look_relative(
                keyboard_input,
                controls,
                config,
                mouse_motion,
                camera_query,
            );
        }
        LookMode::Absolute => {
            handle_camera_look_absolute(
                keyboard_input,
                controls,
                config,
                camera_query,
                cursor_query,
            );
        }
    }
}

pub fn toggle_camera_look_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
    mut look_mode: ResMut<CameraLookMode>,
) {
    if keyboard_input.just_pressed(controls.look_mode_toggle) {
        look_mode.0 = match look_mode.0 {
            LookMode::Relative => LookMode::Absolute,
            LookMode::Absolute => LookMode::Relative,
        };
    }
}

fn handle_camera_look_relative(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
    config: Res<GameConfig>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut CameraLook), With<Camera3d>>,
) {
    let mouse_sensitivity = config.camera.mouse_sensitivity;
    let max_horizontal = config.camera.max_look_horizontal.to_radians();
    let max_up = config.camera.max_look_up.to_radians();
    let max_down = config.camera.max_look_down.to_radians();

    let is_looking = keyboard_input.pressed(controls.look_hold);

    let mut total_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        total_delta += event.delta;
    }

    for (mut transform, mut camera_look) in &mut camera_query {
        if is_looking {
            if total_delta != Vec2::ZERO {
                let yaw_delta = -total_delta.x * mouse_sensitivity * 0.01;
                let pitch_delta = -total_delta.y * mouse_sensitivity * 0.01;

                camera_look.target_yaw += yaw_delta;
                camera_look.target_pitch += pitch_delta;

                camera_look.target_yaw =
                    camera_look.target_yaw.clamp(-max_horizontal, max_horizontal);

                camera_look.target_pitch = camera_look.target_pitch.clamp(-max_down, max_up);
            }
        } else {
            camera_look.target_yaw = 0.0;
            camera_look.target_pitch = 0.0;
        }

        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);

        transform.rotation = yaw_quat * pitch_quat;
    }
}

fn handle_camera_look_absolute(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
    config: Res<GameConfig>,
    mut camera_query: Query<(&mut Transform, &mut CameraLook), With<Camera3d>>,
    cursor_query: Query<&Window, With<PrimaryWindow>>,
) {
    let max_horizontal = config.camera.max_look_horizontal.to_radians();
    let max_up = config.camera.max_look_up.to_radians();
    let max_down = config.camera.max_look_down.to_radians();

    let is_looking = keyboard_input.pressed(controls.look_hold);

    let window = cursor_query.single().ok();
    let cursor_position = window.and_then(|w: &Window| w.cursor_position());

    for (mut transform, mut camera_look) in &mut camera_query {
        if is_looking {
            if let Some(cursor_pos) = cursor_position {
                if let Some(window) = window {
                    let window_size = Vec2::new(window.resolution.width(), window.resolution.height());
                    let center = window_size / 2.0;

                    let cursor_offset = cursor_pos - center;

                    camera_look.target_yaw =
                        -(cursor_offset.x / (window_size.x / 2.0)) * max_horizontal;

                    let normalized_y = cursor_offset.y / (window_size.y / 2.0);
                    if normalized_y > 0.0 {
                        camera_look.target_pitch = -normalized_y * max_down;
                    } else {
                        camera_look.target_pitch = -normalized_y * max_up;
                    }

                    camera_look.target_yaw =
                        camera_look.target_yaw.clamp(-max_horizontal, max_horizontal);
                    camera_look.target_pitch = camera_look.target_pitch.clamp(-max_down, max_up);
                }
            }
        } else {
            camera_look.target_yaw = 0.0;
            camera_look.target_pitch = 0.0;
        }

        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);

        transform.rotation = yaw_quat * pitch_quat;
    }
}

pub fn update_camera_look_lerp(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut camera_query: Query<(&mut Transform, &mut CameraLook), With<Camera3d>>,
) {
    let look_lerp_speed = config.camera.look_lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut camera_look) in &mut camera_query {
        let lerp_factor = (look_lerp_speed * delta_time).min(1.0);
        camera_look.yaw = camera_look.yaw.lerp(camera_look.target_yaw, lerp_factor);
        camera_look.pitch = camera_look.pitch.lerp(camera_look.target_pitch, lerp_factor);

        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);
        transform.rotation = yaw_quat * pitch_quat;
    }
}
