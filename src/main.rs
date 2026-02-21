use bevy::prelude::*;
use bevy::ui::{Node, PositionType, Val};
use bevy::log::LogPlugin;
use serde::Deserialize;
use std::fs;

mod system;
mod map;

use system::camera::{
    handle_camera_look, toggle_camera_look_mode, update_camera_look_lerp, CameraLook,
};
use system::movement::{
    handle_player_input, update_lerp_movement, update_lerp_rotation,
    Collider, LerpMovement, LerpRotation, InputRepeatTimer, MovementState,
};
use map::{CellGraph, spawn_cell_walls};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LookMode {
    Relative,
    Absolute,
}

#[derive(Resource, Clone, Debug)]
struct CameraLookMode(LookMode);

#[derive(Deserialize, Clone)]
pub struct PlayerConfig {
    pub grid_unit: f32,
    pub lerp_speed: f32,
    pub rotation_lerp_speed: f32,
    pub input_repeat_delay: f32,
    /// Total height of the player creature in world units.
    pub creature_height: f32,
    /// Width (and depth) of the player creature's collider in world units.
    pub creature_width: f32,
    /// Height above the creature's feet where the camera (eyes) sits.
    pub eye_height: f32,
}

#[derive(Deserialize, Clone)]
pub struct CameraConfig {
    pub look_mode: LookMode,
    pub mouse_sensitivity: f32,
    pub max_look_horizontal: f32,
    pub max_look_up: f32,
    pub max_look_down: f32,
    pub look_lerp_speed: f32,
}

#[derive(Deserialize, Clone)]
pub struct ControlsConfig {
    pub move_forward: String,
    pub move_backward: String,
    pub move_left: String,
    pub move_right: String,
    pub rotate_left: String,
    pub rotate_right: String,
    pub look_hold: String,
    pub look_mode_toggle: String,
}

#[derive(Deserialize, Clone)]
pub struct DebugConfig {
    pub log_level: String,
}

#[derive(Deserialize, Clone)]
pub struct GameConfig {
    pub player: PlayerConfig,
    pub camera: CameraConfig,
    pub controls: ControlsConfig,
    pub debug: DebugConfig,
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

fn parse_log_level(value: &str) -> bevy::log::Level {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" => bevy::log::Level::ERROR,
        "error" => bevy::log::Level::ERROR,
        "warn" => bevy::log::Level::WARN,
        "info" => bevy::log::Level::INFO,
        "debug" => bevy::log::Level::DEBUG,
        "trace" => bevy::log::Level::TRACE,
        _ => {
            eprintln!("Invalid log level '{}', defaulting to 'info'", value);
            bevy::log::Level::INFO
        }
    }
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

    let log_level = parse_log_level(&config.debug.log_level);

    // Configure log filter to suppress noisy third-party crates
    let log_filter = format!(
        "{}={},wgpu=warn,naga=warn,cosmic_text=info",
        env!("CARGO_PKG_NAME").replace("-", "_"),
        config.debug.log_level.to_lowercase()
    );

    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: log_level,
            filter: log_filter,
            ..default()
        }))
        .insert_resource(config.clone())
        .insert_resource(controls)
        .insert_resource(CameraLookMode(config.camera.look_mode.clone()))
        .insert_resource(CellGraph::new(5.0))
        .add_systems(Startup, (setup, setup_corridor, spawn_cell_walls).chain())
        .add_systems(Update, (handle_player_input, update_lerp_movement, update_lerp_rotation))
        .add_systems(Update, (toggle_camera_look_mode, handle_camera_look, update_camera_look_lerp))
        .add_systems(Update, update_debug_text)
        .run();
}

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    let creature_width = config.player.creature_width;
    let creature_height = config.player.creature_height;
    let eye_height = config.player.eye_height;

    // Player entity (parent) - handles movement, rotation, and collision
    let player = commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(1.0, 0.0, 0.0), Vec3::Y), 
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: Vec3::new(0.0, 0.0, 0.0),
            start_position: Vec3::new(0.0, 0.0, 0.0),
            lerp_progress: 0.0,
        },
        LerpRotation {
            rotation_delta: 0.0,
            target_rotation: Quat::from_rotation_y(-90.0_f32.to_radians()),
            lerp_progress: 0.0,
        },
        InputRepeatTimer {
            movement_timer: 0.0,
            rotation_timer: 0.0,
        },
        Collider {
            size: Vec3::new(creature_width, creature_height, creature_width),
        },
    )).id();

    // Camera entity (child) - positioned at eye height, inherits parent rotation
    commands.spawn((
        Camera3d::default(),
        CameraLook {
            yaw: 0.0,
            pitch: 0.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
        },
        Transform::from_xyz(0.0, eye_height, 0.0),
    )).set_parent_in_place(player);

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

fn setup_corridor(
    mut commands: Commands,
    mut graph: ResMut<CellGraph>,
) {
    use map::Direction;

    let cell_size = graph.cell_size();

    // Create 5 cells: 3 in a straight line (East), then 2 branching (North and South)
    let cell_0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
    let cell_1 = graph.add_cell(Vec3::new(cell_size, 0.0, 0.0));
    let cell_2 = graph.add_cell(Vec3::new(cell_size * 2.0, 0.0, 0.0));
    let cell_3 = graph.add_cell(Vec3::new(cell_size * 2.0, 0.0, cell_size));  // North from cell_2
    let cell_4 = graph.add_cell(Vec3::new(cell_size * 2.0, 0.0, -cell_size)); // South from cell_2

    // Connect the corridor: 0 -> 1 -> 2, then 2 -> 3 and 2 -> 4
    graph.connect_cells(cell_0, Direction::East, cell_1);
    graph.connect_cells(cell_1, Direction::East, cell_2);
    graph.connect_cells(cell_2, Direction::North, cell_3);
    graph.connect_cells(cell_2, Direction::South, cell_4);

    // Spawn a weak point light in each cell
    for cell in graph.cells() {
        let position = cell.position();
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                intensity: 500_000.0,  // Weaker than before
                range: 10.0,
                ..default()
            },
            Transform::from_xyz(position.x, position.y + 2.0, position.z), // 2 units above cell center
        ));
    }
}

fn update_debug_text(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&CameraLook, With<Camera3d>>,
    look_mode: Res<CameraLookMode>,
    config: Res<GameConfig>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(camera_look) = camera_query.single() else {
        return;
    };

    let mut debug_text = match debug_text_query.single_mut() {
        Ok(text) => text,
        Err(_) => return,
    };

    let pos = player_transform.translation;
    let forward = player_transform.forward();

    let look_mode_str = match look_mode.0 {
        LookMode::Relative => "Relative",
        LookMode::Absolute => "Absolute",
    };

    debug_text.0 = format!(
        "CONTROLS:\n\
         {} - Look | {} - Toggle Look Mode\n\
         \n\
         LOOK MODE: {}\n\
         \n\
         PLAYER:\n\
         Pos: ({:.2}, {:.2}, {:.2})\n\
         Facing: ({:.2}, {:.2}, {:.2})\n\
         Yaw: {:.2} degrees | Pitch: {:.2} degrees",
        config.controls.look_hold,
        config.controls.look_mode_toggle,
        look_mode_str,
        pos.x, pos.y, pos.z,
        forward.x, forward.y, forward.z,
        (camera_look.yaw).to_degrees(),
        (camera_look.pitch).to_degrees(),
    );
}
