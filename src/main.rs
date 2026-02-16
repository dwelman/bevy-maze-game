use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::ecs::message::MessageReader;
use bevy::window::PrimaryWindow;
use bevy::ui::{Node, PositionType, Val};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum LookMode {
    Relative,
    Absolute,
}

#[derive(Resource, Clone, Debug)]
struct CameraLookMode(LookMode);

#[derive(Deserialize, Clone)]
struct PlayerConfig {
    grid_unit: f32,
    lerp_speed: f32,
    rotation_lerp_speed: f32,
    input_repeat_delay: f32,
}

#[derive(Deserialize, Clone)]
struct CameraConfig {
    look_mode: LookMode,
    mouse_sensitivity: f32,
    max_look_horizontal: f32,
    max_look_up: f32,
    max_look_down: f32,
    look_lerp_speed: f32,
}

#[derive(Deserialize, Clone)]
struct ControlsConfig {
    move_forward: String,
    move_backward: String,
    move_left: String,
    move_right: String,
    rotate_left: String,
    rotate_right: String,
    look_hold: String,
    look_mode_toggle: String,
}

#[derive(Deserialize, Clone)]
struct GameConfig {
    player: PlayerConfig,
    camera: CameraConfig,
    controls: ControlsConfig,
}

impl Resource for GameConfig {}

#[derive(Resource, Clone)]
struct Controls {
    move_forward: KeyCode,
    move_backward: KeyCode,
    move_left: KeyCode,
    move_right: KeyCode,
    rotate_left: KeyCode,
    rotate_right: KeyCode,
    look_hold: KeyCode,
    look_mode_toggle: KeyCode,
}

impl Controls {
    fn from_config(config: &ControlsConfig) -> Result<Self, String> {
        Ok(Self {
            move_forward: parse_keycode(&config.move_forward)?,
            move_backward: parse_keycode(&config.move_backward)?,
            move_left: parse_keycode(&config.move_left)?,
            move_right: parse_keycode(&config.move_right)?,
            rotate_left: parse_keycode(&config.rotate_left)?,
            rotate_right: parse_keycode(&config.rotate_right)?,
            look_hold: parse_keycode(&config.look_hold)?,
            look_mode_toggle: parse_keycode(&config.look_mode_toggle)?,
        })
    }
}

fn parse_keycode(value: &str) -> Result<KeyCode, String> {
    let normalized = value.trim().to_ascii_lowercase();
    let key = match normalized.as_str() {
        "w" | "keyw" | "key_w" => KeyCode::KeyW,
        "a" | "keya" | "key_a" => KeyCode::KeyA,
        "s" | "keys" | "key_s" => KeyCode::KeyS,
        "d" | "keyd" | "key_d" => KeyCode::KeyD,
        "q" | "keyq" | "key_q" => KeyCode::KeyQ,
        "e" | "keye" | "key_e" => KeyCode::KeyE,
        "m" | "keym" | "key_m" => KeyCode::KeyM,
        "tab" => KeyCode::Tab,
        "arrowup" | "arrow_up" | "up" => KeyCode::ArrowUp,
        "arrowdown" | "arrow_down" | "down" => KeyCode::ArrowDown,
        "arrowleft" | "arrow_left" | "left" => KeyCode::ArrowLeft,
        "arrowright" | "arrow_right" | "right" => KeyCode::ArrowRight,
        "pageup" | "page_up" => KeyCode::PageUp,
        "pagedown" | "page_down" => KeyCode::PageDown,
        _ => {
            return Err(format!(
                "Unsupported key '{}'. Use W/A/S/D, Q/E, Tab, M, Arrow keys, or PageUp/PageDown.",
                value
            ));
        }
    };

    Ok(key)
}

fn main() {
    // Load config
    let config_path = "config.toml";
    let config_str = fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", config_path));
    let config: GameConfig = toml::from_str(&config_str)
        .expect("Failed to parse config file");

    let controls = Controls::from_config(&config.controls)
        .unwrap_or_else(|err| panic!("Invalid controls in config.toml: {}", err));

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config.clone())
        .insert_resource(controls)
        .insert_resource(CameraLookMode(config.camera.look_mode.clone()))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_player_input, update_lerp_movement, update_lerp_rotation))
        .add_systems(Update, (toggle_camera_look_mode, handle_camera_look, update_camera_look_lerp))
        .add_systems(Update, update_debug_text)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct LerpMovement {
    /// The target position the player is moving towards
    target_position: Vec3,
    /// Current lerp progress from 0.0 to 1.0
    lerp_progress: f32,
}

#[derive(Component)]
struct LerpRotation {
    /// The target rotation the player is rotating towards
    target_rotation: Quat,
    /// Current lerp progress from 0.0 to 1.0
    lerp_progress: f32,
}

#[derive(Component)]
struct InputRepeatTimer {
    /// Tracks time since last movement input
    movement_timer: f32,
    /// Tracks time since last rotation input
    rotation_timer: f32,
}

#[derive(Component)]
struct CameraLook {
    /// Current horizontal look angle in radians (positive = right, negative = left)
    yaw: f32,
    /// Current vertical look angle in radians (positive = up, negative = down)
    pitch: f32,
    /// Target horizontal look angle we're lerping towards
    target_yaw: f32,
    /// Target vertical look angle we're lerping towards
    target_pitch: f32,
}

#[derive(Component)]
struct DebugText;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    // player entity with camera
    let player_pos = Vec3::new(0.0, 0.5, 4.5);
    commands.spawn((
        Player,
        LerpMovement {
            target_position: player_pos,
            lerp_progress: 1.0, // Start at target so no initial lerp
        },
        LerpRotation {
            target_rotation: Quat::IDENTITY,
            lerp_progress: 1.0, // Start at target so no initial lerp
        },
        InputRepeatTimer {
            movement_timer: 0.0,
            rotation_timer: 0.0,
        },
        Transform::from_translation(player_pos),
    )).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            CameraLook {
                yaw: 0.0,
                pitch: 0.0,
                target_yaw: 0.0,
                target_pitch: 0.0,
            },
            Transform::IDENTITY.looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });

    // Debug text UI
    commands.spawn((
        Text::new(""),
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        DebugText,
    ));
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    controls: Res<Controls>,
    mut player_query: Query<(&Transform, &mut LerpMovement, &mut LerpRotation, &mut InputRepeatTimer), With<Player>>,
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
            if keyboard_input.just_pressed(controls.move_forward) || input_timer.movement_timer >= repeat_delay {
                movement_delta += forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_backward) {
            if keyboard_input.just_pressed(controls.move_backward) || input_timer.movement_timer >= repeat_delay {
                movement_delta -= forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_left) {
            if keyboard_input.just_pressed(controls.move_left) || input_timer.movement_timer >= repeat_delay {
                movement_delta -= right * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.move_right) {
            if keyboard_input.just_pressed(controls.move_right) || input_timer.movement_timer >= repeat_delay {
                movement_delta += right * grid_unit;
                movement_triggered = true;
            }
        }

        // Check rotation inputs
        if keyboard_input.pressed(controls.rotate_left) {
            if keyboard_input.just_pressed(controls.rotate_left) || input_timer.rotation_timer >= repeat_delay {
                rotation_delta = std::f32::consts::FRAC_PI_2; // 90 degrees
                rotation_triggered = true;
            }
        }
        if keyboard_input.pressed(controls.rotate_right) {
            if keyboard_input.just_pressed(controls.rotate_right) || input_timer.rotation_timer >= repeat_delay {
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

fn update_lerp_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &mut LerpMovement), With<Player>>,
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

fn update_lerp_rotation(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<(&mut Transform, &mut LerpRotation), With<Player>>,
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

fn handle_camera_look(
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

fn toggle_camera_look_mode(
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

    // Check if look key (Tab) is pressed
    let is_looking = keyboard_input.pressed(controls.look_hold);

    // Accumulate mouse motion
    let mut total_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        total_delta += event.delta;
    }

    for (mut transform, mut camera_look) in &mut camera_query {
        if is_looking {
            // Only update target angles if Tab is held
            if total_delta != Vec2::ZERO {
                let yaw_delta = -total_delta.x * mouse_sensitivity * 0.01;
                let pitch_delta = -total_delta.y * mouse_sensitivity * 0.01;

                camera_look.target_yaw += yaw_delta;
                camera_look.target_pitch += pitch_delta;

                // Clamp horizontal look (yaw)
                camera_look.target_yaw = camera_look.target_yaw.clamp(-max_horizontal, max_horizontal);

                // Clamp vertical look (pitch)
                camera_look.target_pitch = camera_look.target_pitch.clamp(-max_down, max_up);
            }
        } else {
            // Tab not held - start returning to center
            camera_look.target_yaw = 0.0;
            camera_look.target_pitch = 0.0;
        }

        // Build rotation from yaw and pitch
        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);

        // Combine rotations: yaw first, then pitch
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

    // Check if look key (Tab) is pressed
    let is_looking = keyboard_input.pressed(controls.look_hold);

    let window = cursor_query.single().ok();
    let cursor_position = window.and_then(|w: &Window| w.cursor_position());

    for (mut transform, mut camera_look) in &mut camera_query {
        if is_looking {
            if let Some(cursor_pos) = cursor_position {
                if let Some(window) = window {
                    let window_size = Vec2::new(window.resolution.width(), window.resolution.height());
                    let center = window_size / 2.0;

                    // Normalize cursor position relative to center
                    let cursor_offset = cursor_pos - center;

                    // Map to angle range
                    // Horizontal: map from [-width/2, width/2] to [-max_horizontal, max_horizontal]
                    camera_look.target_yaw = -(cursor_offset.x / (window_size.x / 2.0)) * max_horizontal;

                    // Vertical: map from [-height/2, height/2] to [-max_down, max_up]
                    // (inverted because screen y increases downward)
                    let normalized_y = cursor_offset.y / (window_size.y / 2.0);
                    if normalized_y > 0.0 {
                        // Looking down
                        camera_look.target_pitch = -normalized_y * max_down;
                    } else {
                        // Looking up
                        camera_look.target_pitch = -normalized_y * max_up;
                    }

                    // Clamp to ensure we don't exceed bounds
                    camera_look.target_yaw = camera_look.target_yaw.clamp(-max_horizontal, max_horizontal);
                    camera_look.target_pitch = camera_look.target_pitch.clamp(-max_down, max_up);
                }
            }
        } else {
            // Tab not held - return to center
            camera_look.target_yaw = 0.0;
            camera_look.target_pitch = 0.0;
        }

        // Build rotation from yaw and pitch
        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);

        // Combine rotations: yaw first, then pitch
        transform.rotation = yaw_quat * pitch_quat;
    }
}

fn update_camera_look_lerp(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut camera_query: Query<(&mut Transform, &mut CameraLook), With<Camera3d>>,
) {
    let look_lerp_speed = config.camera.look_lerp_speed;
    let delta_time = time.delta_secs();

    for (mut transform, mut camera_look) in &mut camera_query {
        // Smoothly lerp towards target angles
        let lerp_factor = (look_lerp_speed * delta_time).min(1.0);
        camera_look.yaw = camera_look.yaw.lerp(camera_look.target_yaw, lerp_factor);
        camera_look.pitch = camera_look.pitch.lerp(camera_look.target_pitch, lerp_factor);

        // Update rotation
        let yaw_quat = Quat::from_rotation_y(camera_look.yaw);
        let pitch_quat = Quat::from_rotation_x(camera_look.pitch);
        transform.rotation = yaw_quat * pitch_quat;
    }
}

fn update_debug_text(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<(&Transform, &CameraLook), With<Camera3d>>,
    look_mode: Res<CameraLookMode>,
    config: Res<GameConfig>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok((_camera_transform, camera_look)) = camera_query.single() else {
        return;
    };

    let mut debug_text = match debug_text_query.single_mut() {
        Ok(text) => text,
        Err(_) => return,
    };

    let pos = player_transform.translation;
    let rot = player_transform.rotation;
    let euler = rot.to_euler(bevy::math::EulerRot::YXZ);
    
    let look_mode_str = match look_mode.0 {
        LookMode::Relative => "Relative",
        LookMode::Absolute => "Absolute",
    };

    debug_text.0 = format!(
        "CONTROLS:\n\
         {} / {} / {} / {} - Move | {} / {} - Rotate | {} - Look | {} - Toggle Look Mode\n\
         \n\
         LOOK MODE: {}\n\
         \n\
         PLAYER:\n\
         Pos: ({:.2}, {:.2}, {:.2})\n\
         Rot: ({:.2}, {:.2}, {:.2})\n\
         \n\
         CAMERA:\n\
         Yaw: {:.2} degrees | Pitch: {:.2} degrees",
        config.controls.move_forward,
        config.controls.move_left,
        config.controls.move_backward,
        config.controls.move_right,
        config.controls.rotate_left,
        config.controls.rotate_right,
        config.controls.look_hold,
        config.controls.look_mode_toggle,
        look_mode_str,
        pos.x, pos.y, pos.z,
        euler.0.to_degrees(), euler.1.to_degrees(), euler.2.to_degrees(),
        (camera_look.yaw).to_degrees(),
        (camera_look.pitch).to_degrees(),
    );
}
