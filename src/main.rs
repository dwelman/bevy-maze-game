use bevy::prelude::*;
use bevy::ui::{Node, PositionType, Val};
use bevy::log::LogPlugin;
use serde::Deserialize;
use std::fs;

mod map;
mod system;

use map::{Direction, Edge, EdgeType, Grid};
use system::camera::{
    handle_camera_look, toggle_camera_look_mode, update_camera_look_lerp, CameraLook,
};
use system::movement::{
    handle_player_input, update_lerp_movement, update_lerp_rotation, Collider,
    InputRepeatTimer, LerpMovement, LerpRotation, MovementState,
};

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
    pub collider_size: [f32; 3],
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

/// Room dimensions in grid cells.
#[derive(Resource)]
struct RoomConfig {
    width: i32,
    depth: i32,
    height: i32,
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
        .insert_resource(RoomConfig { width: 10, depth: 10, height: 5 })
        .add_systems(Startup, (setup, spawn_room))
        .add_systems(Update, (handle_player_input, update_lerp_movement, update_lerp_rotation))
        .add_systems(Update, (toggle_camera_look_mode, handle_camera_look, update_camera_look_lerp))
        .add_systems(Update, update_debug_text)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugText;

/// Builds the room grid and spawns all wall/floor/ceiling geometry in one pass.
///
/// Deduplication rule — for each shared interior edge, only one panel is spawned:
/// - East, South, Up faces: always spawn if solid (positive-axis canonical owner)
/// - West, North, Down faces: only spawn at boundary (no neighbour in that direction)
fn spawn_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
    room: Res<RoomConfig>,
) {
    let g = config.player.grid_unit;
    let (width, depth, height) = (room.width, room.depth, room.height);
    const WALL_T: f32 = 0.1;

    let mut grid = Grid::new();

    // Phase 1: add all cells (wires neighbour links bidirectionally).
    for y in 0..height {
        for z in 0..depth {
            for x in 0..width {
                grid.add_cell(x, y, z);
            }
        }
    }

    // Phase 2: open interior edges.
    // Each set_edge call mirrors to the neighbour automatically.
    for y in 0..height {
        for z in 0..depth {
            for x in 0..width {
                let coord = IVec3::new(x, y, z);

                // Vertical: floor on bottom layer, ceiling on top, open in between.
                if y == 0 {
                    grid.set_edge(coord, Direction::Down, Edge::floor());
                } else {
                    grid.set_edge(coord, Direction::Down, Edge::open());
                }
                if y == height - 1 {
                    grid.set_edge(coord, Direction::Up, Edge::ceiling());
                }
                // Note: interior Up edges default to Wall until the cell above
                // sets its Down to open (mirrored), so no explicit Up open needed here.

                // Horizontal: open interior connections (avoid double-opening by
                // only handling East and South — each shared edge covered once).
                if x < width - 1 {
                    grid.set_edge(coord, Direction::East, Edge::open());
                }
                if z < depth - 1 {
                    grid.set_edge(coord, Direction::South, Edge::open());
                }
            }
        }
    }

    // Phase 3: spawn geometry for every solid edge (deduplicated).
    let wall_mat  = materials.add(Color::srgb(0.45, 0.45, 0.55));
    let floor_mat = materials.add(Color::srgb(0.30, 0.28, 0.25));
    let ceil_mat  = materials.add(Color::srgb(0.25, 0.25, 0.30));

    // Pre-create mesh handles — all panels of the same shape share one GPU mesh.
    let ns_mesh = meshes.add(Cuboid::new(g, g, WALL_T)); // North/South face
    let ew_mesh = meshes.add(Cuboid::new(WALL_T, g, g)); // East/West face
    let h_mesh  = meshes.add(Cuboid::new(g, WALL_T, g)); // Floor/Ceiling

    // Collect coords first to avoid borrow conflict when reading cells + inserting resource.
    let coords: Vec<IVec3> = grid.cells.keys().copied().collect();

    for coord in coords {
        let cell = &grid.cells[&coord];
        let cx = coord.x as f32 * g;
        let cy = coord.y as f32 * g;
        let cz = coord.z as f32 * g;
        let half = g * 0.5;

        // ── East (+X) — always spawn if Wall ────────────────────────────────
        if cell.get_edge(Direction::East).edge_type == EdgeType::Wall {
            commands.spawn((
                Mesh3d(ew_mesh.clone()),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(cx + half, cy, cz),
                Collider { size: Vec3::new(WALL_T, g, g) },
            ));
        }

        // ── South (+Z) — always spawn if Wall ───────────────────────────────
        if cell.get_edge(Direction::South).edge_type == EdgeType::Wall {
            commands.spawn((
                Mesh3d(ns_mesh.clone()),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(cx, cy, cz + half),
                Collider { size: Vec3::new(g, g, WALL_T) },
            ));
        }

        // ── Up (ceiling) — always spawn if Ceiling ──────────────────────────
        if cell.get_edge(Direction::Up).edge_type == EdgeType::Ceiling {
            commands.spawn((
                Mesh3d(h_mesh.clone()),
                MeshMaterial3d(ceil_mat.clone()),
                Transform::from_xyz(cx, cy + half, cz),
                Collider { size: Vec3::new(g, WALL_T, g) },
            ));
        }

        // ── West (-X) — boundary only (no West neighbour) ───────────────────
        if cell.get_edge(Direction::West).edge_type == EdgeType::Wall
            && cell.neighbours.get(&Direction::West).copied().flatten().is_none()
        {
            commands.spawn((
                Mesh3d(ew_mesh.clone()),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(cx - half, cy, cz),
                Collider { size: Vec3::new(WALL_T, g, g) },
            ));
        }

        // ── North (-Z) — boundary only (no North neighbour) ─────────────────
        if cell.get_edge(Direction::North).edge_type == EdgeType::Wall
            && cell.neighbours.get(&Direction::North).copied().flatten().is_none()
        {
            commands.spawn((
                Mesh3d(ns_mesh.clone()),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(cx, cy, cz - half),
                Collider { size: Vec3::new(g, g, WALL_T) },
            ));
        }

        // ── Down (floor) — boundary only (no Down neighbour) ────────────────
        if cell.get_edge(Direction::Down).edge_type == EdgeType::Floor
            && cell.neighbours.get(&Direction::Down).copied().flatten().is_none()
        {
            commands.spawn((
                Mesh3d(h_mesh.clone()),
                MeshMaterial3d(floor_mat.clone()),
                Transform::from_xyz(cx, cy - half, cz),
                Collider { size: Vec3::new(g, WALL_T, g) },
            ));
        }
    }

    commands.insert_resource(grid);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 2_000_000.0,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(2.0, 4.0, 2.0),
    ));

    // Player — spawns at cell (2, 0, 2).
    // Cell centres are at integer world coords; player Y = half body height.
    let player_pos = Vec3::new(2.0, 0.975, 2.0);

    commands.spawn((
        Player,
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: player_pos,
            start_position: player_pos,
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

        // Camera at eye level
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
