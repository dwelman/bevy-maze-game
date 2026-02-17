use bevy::prelude::*;
use bevy::ui::{Node, PositionType, Val};
use serde::Deserialize;
use std::fs;

mod system;

use system::camera::{
    handle_camera_look, toggle_camera_look_mode, update_camera_look_lerp, CameraLook,
};
use system::movement::{
    handle_player_input, update_lerp_movement, update_lerp_rotation, Collider,
    InputRepeatTimer, LerpMovement, LerpRotation, MovementState,
};

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
    collider_size: [f32; 3],
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
struct DebugConfig {
    logging: bool,
}

#[derive(Deserialize, Clone)]
struct GameConfig {
    player: PlayerConfig,
    camera: CameraConfig,
    controls: ControlsConfig,
    debug: DebugConfig,
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
struct DebugText;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // Create some obstacles to test collision
    // Center cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Collider {
            size: Vec3::new(1.0, 1.0, 1.0),
        },
    ));
    
    // Wall in front of player
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 200))),
        Transform::from_xyz(0.0, 1.0, 3.0),
        Collider {
            size: Vec3::new(1.0, 2.0, 1.0),
        },
    ));
    
    // corner walls
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 200))),
        Transform::from_xyz(2.0, 1.0, 4.0),
        Collider {
            size: Vec3::new(1.0, 2.0, 1.0),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 200))),
        Transform::from_xyz(1.0, 1.0, 4.0),
        Collider {
            size: Vec3::new(1.0, 2.0, 1.0),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 100, 200))),
        Transform::from_xyz(1.0, 1.0, 3.0),
        Collider {
            size: Vec3::new(1.0, 2.0, 1.0),
        },
    ));

    // Smaller hitbox obstacles
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 1.65, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 20, 20))),
        Transform::from_xyz(1.0, 1.0, -4.0),
        Collider {
            size: Vec3::new(0.8, 1.65, 0.8),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 1.65, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 20, 20))),
        Transform::from_xyz(2.0, 1.0, -4.0),
        Collider {
            size: Vec3::new(0.8, 1.65, 0.8),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.8, 1.65, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb_u8(100, 20, 20))),
        Transform::from_xyz(2.0, 1.0, -3.0),
        Collider {
            size: Vec3::new(0.8, 1.65, 0.8),
        },
    ));

    // Add gap the player can fit through to test collision edge cases
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.3, 2.0, 1.3))),
        MeshMaterial3d(materials.add(Color::srgb_u8(20, 100, 20))),
        Transform::from_xyz(-4.0, 1.0, 0.0),
        Collider {
            size: Vec3::new(1.3, 2.0, 1.3),
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.3, 2.0, 1.3))),
        MeshMaterial3d(materials.add(Color::srgb_u8(20, 100, 20))),
        Transform::from_xyz(-4.0, 1.0, 2.0),
        Collider {
            size: Vec3::new(1.3, 2.0, 1.3),
        },
    ));

    // Add planes to test collision with flat surfaces
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 4.0, 0.1))),
        MeshMaterial3d(materials.add(Color::srgb_u8(20, 20, 100))),
        Transform::from_xyz(0.0, 2.0, -4.0),
        Collider {
            size: Vec3::new(4.0, 4.0, 0.1),
        },
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
    // Player body is 0.75 x 0.75 x 1.75, fits in 1x1x2 grid space
    // Position at y = 0.875 (half of 1.75) to start on ground
    let player_pos = Vec3::new(0.0, 0.875, 5.0);
    commands.spawn((
        Player,
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: player_pos,
            start_position: player_pos,
            lerp_progress: 1.0, // Start at target so no initial lerp
        },
        LerpRotation {
            rotation_delta: 0.0,
            target_rotation: Quat::IDENTITY,
            lerp_progress: 1.0, // Start at target so no initial lerp
        },
        InputRepeatTimer {
            movement_timer: 0.0,
            rotation_timer: 0.0,
        },
        Collider {
            size: Vec3::from_slice(&config.player.collider_size),
        },
        Transform::from_translation(player_pos),
    )).with_children(|parent| {
        // Player body mesh
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.75, 1.75, 0.75))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::IDENTITY,
        ));
        
        // Camera positioned at eye level (near top of player body)
        parent.spawn((
            Camera3d::default(),
            CameraLook {
                yaw: 0.0,
                pitch: 0.0,
                target_yaw: 0.0,
                target_pitch: 0.0,
            },
            Transform::from_xyz(0.0, 0.7, 0.0).looking_at(Vec3::new(0.0, 0.7, -1.0), Vec3::Y),
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
