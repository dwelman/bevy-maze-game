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
    handle_player_input, update_lerp_rotation, update_cell_movement,
    CellTransform, LerpMovement, LerpRotation, InputRepeatTimer, MovementState,
};
use map::{CellGraph, CellId, Direction, RoomMap, spawn_cell_walls};

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
    pub lerp_speed: f32,
    pub rotation_lerp_speed: f32,
    pub input_repeat_delay: f32,
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
pub struct WorldConfig {
    pub cell_size: f32,
}

#[derive(Deserialize, Clone)]
pub struct GameConfig {
    pub player: PlayerConfig,
    pub camera: CameraConfig,
    pub controls: ControlsConfig,
    pub debug: DebugConfig,
    pub world: WorldConfig,
}

impl Resource for GameConfig {}

#[derive(Resource, Default)]
pub struct GoalCellId(pub Option<CellId>);

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
        .insert_resource(CellGraph::new(config.world.cell_size))
        .insert_resource(RoomMap::new())
        .insert_resource(DebugVisible(false))
        .insert_resource(WinState(false))
        .init_resource::<GoalCellId>()
        .add_systems(Startup, (setup_rooms, setup_goal_cell, setup, spawn_cell_walls).chain())
        .add_systems(Update, (handle_player_input, update_cell_movement, update_lerp_rotation))
        .add_systems(Update, (toggle_camera_look_mode, handle_camera_look, update_camera_look_lerp))
        .add_systems(Update, (toggle_debug_text, update_debug_text))
        .add_systems(Update, check_win_condition)
        .run();
}

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct WinText;

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct DebugVisible(bool);

#[derive(Resource)]
struct WinState(bool);

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
    graph: Res<CellGraph>,
) {
    let eye_height = config.player.eye_height;

    // Find the starting cell (cell at position 0,0,0)
    let starting_cell = graph.cells()
        .find(|cell| {
            let pos = cell.position();
            pos.x == 0.0 && pos.z == 0.0
        })
        .expect("Starting cell not found");

    let player_y = starting_cell.position().y;
    let floor_height = graph.get_cell_floor_height(starting_cell.id()).unwrap();

    // Player entity (parent) - handles movement, rotation, and collision
    let initial_facing = Direction::North;
    let initial_rotation = initial_facing.to_quat();
    let player = commands.spawn((
        Player,
        Transform::from_xyz(0.0, player_y, 0.0).with_rotation(initial_rotation),
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: Vec3::new(0.0, player_y, 0.0),
            start_position: Vec3::new(0.0, player_y, 0.0),
            lerp_progress: 0.0,
            pending_cell: None,
        },
        LerpRotation {
            rotation_delta: 0.0,
            target_rotation: initial_rotation,
            lerp_progress: 1.0,
        },
        InputRepeatTimer {
            movement_timer: 0.0,
            rotation_timer: 0.0,
        },
        CellTransform {
            cell: starting_cell.id(),
            facing: initial_facing,
        },
    )).id();

    // Camera entity (child) - sits at eye_height above the cell floor, relative to player
    let camera_y_relative = (floor_height + eye_height) - player_y;
    commands.spawn((
        Camera3d::default(),
        CameraLook {
            yaw: 0.0,
            pitch: 0.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
        },
        Transform::from_xyz(0.0, camera_y_relative, 0.0),
        ChildOf(player),
    ));

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
        Visibility::Hidden,
    ));

    // Win message UI - centered
    commands.spawn((
        Text::new("You found the exit!"),
        TextColor(Color::srgb(1.0, 0.85, 0.0)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            width: Val::Percent(100.0),
            ..default()
        },
        WinText,
        Visibility::Hidden,
    ));
}

fn setup_rooms(
    mut commands: Commands,
    mut graph: ResMut<CellGraph>,
    mut room_map: ResMut<RoomMap>,
) {
    let mut rng = rand::rng();
    let cell_size = graph.cell_size();

    // Create rooms with random colors
    let room_0_id = room_map.create_room(random_room_color(&mut rng));
    let room_1_id = room_map.create_room(random_room_color(&mut rng));
    let room_2_id = room_map.create_room(random_room_color(&mut rng));

    // Room 0: 2x2 square (player starts here at 0,0,0)
    let r0c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
    let r0c1 = graph.add_cell(Vec3::new(cell_size, 0.0, 0.0));
    let r0c2 = graph.add_cell(Vec3::new(0.0, 0.0, -cell_size));
    let r0c3 = graph.add_cell(Vec3::new(cell_size, 0.0, -cell_size));

    graph.connect_cells(r0c0, Direction::East, r0c1);
    graph.connect_cells(r0c0, Direction::North, r0c2);
    graph.connect_cells(r0c1, Direction::North, r0c3);
    graph.connect_cells(r0c2, Direction::East, r0c3);

    room_map.assign_cell(r0c0, room_0_id);
    room_map.assign_cell(r0c1, room_0_id);
    room_map.assign_cell(r0c2, room_0_id);
    room_map.assign_cell(r0c3, room_0_id);

    // Room 1: L-shaped corridor going north then east
    let r1c0 = graph.add_cell(Vec3::new(0.0, 0.0, -cell_size * 2.0));
    let r1c1 = graph.add_cell(Vec3::new(0.0, 0.0, -cell_size * 3.0));
    let r1c2 = graph.add_cell(Vec3::new(cell_size, 0.0, -cell_size * 3.0));

    graph.connect_cells(r1c0, Direction::North, r1c1);
    graph.connect_cells(r1c1, Direction::East, r1c2);

    room_map.assign_cell(r1c0, room_1_id);
    room_map.assign_cell(r1c1, room_1_id);
    room_map.assign_cell(r1c2, room_1_id);

    // Room 2: 2-cell corridor going north, east of room 0
    let r2c0 = graph.add_cell(Vec3::new(cell_size * 2.0, 0.0, 0.0));
    let r2c1 = graph.add_cell(Vec3::new(cell_size * 2.0, 0.0, -cell_size));

    graph.connect_cells(r2c0, Direction::North, r2c1);

    room_map.assign_cell(r2c0, room_2_id);
    room_map.assign_cell(r2c1, room_2_id);

    // Connect rooms via edge cells (both in the graph and in the room map)
    // Room 0 north edge -> Room 1 south edge
    graph.connect_cells(r0c2, Direction::North, r1c0);
    room_map.connect_rooms(room_0_id, r0c2, Direction::North, room_1_id, r1c0);

    // Room 0 east edge -> Room 2 west edge
    graph.connect_cells(r0c1, Direction::East, r2c0);
    room_map.connect_rooms(room_0_id, r0c1, Direction::East, room_2_id, r2c0);

    // Spawn a point light in each cell
    for cell in graph.cells() {
        let position = cell.position();
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                intensity: 500_000.0,
                range: 10.0,
                ..default()
            },
            Transform::from_xyz(position.x, (position.y + (cell_size / 2.0)) - 0.2, position.z),
        ));
    }
}

fn random_room_color(rng: &mut impl rand::Rng) -> Color {
    use rand::RngExt;
    let hue: f32 = rng.random_range(0.0f32..360.0f32);
    Color::hsl(hue, 0.7, 0.4)
}

fn toggle_debug_text(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_visible: ResMut<DebugVisible>,
    mut debug_text_query: Query<&mut Visibility, With<DebugText>>,
) {
    if keyboard_input.just_pressed(KeyCode::Backquote) {
        debug_visible.0 = !debug_visible.0;

        if let Ok(mut visibility) = debug_text_query.single_mut() {
            *visibility = if debug_visible.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn update_debug_text(
    player_query: Query<(&Transform, &CellTransform), With<Player>>,
    camera_query: Query<&CameraLook, With<Camera3d>>,
    look_mode: Res<CameraLookMode>,
    config: Res<GameConfig>,
    debug_visible: Res<DebugVisible>,
    mut debug_text_query: Query<&mut Text, With<DebugText>>,
) {
    if !debug_visible.0 {
        return;
    }

    let Ok((player_transform, cell_tf)) = player_query.single() else {
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

    let look_mode_str = match look_mode.0 {
        LookMode::Relative => "Relative",
        LookMode::Absolute => "Absolute",
    };

    let text = format!(
        "CONTROLS:\n\
         {} - Look | {} - Toggle Look Mode\n\
         \n\
         LOOK MODE: {}\n\
         \n\
         PLAYER:\n\
         Pos: ({:.2}, {:.2}, {:.2})\n\
         Cell: {:?} | Facing: {:?}\n\
         Yaw: {:.2} degrees | Pitch: {:.2} degrees",
        config.controls.look_hold,
        config.controls.look_mode_toggle,
        look_mode_str,
        pos.x, pos.y, pos.z,
        cell_tf.cell, cell_tf.facing,
        (camera_look.yaw).to_degrees(),
        (camera_look.pitch).to_degrees(),
    );

    debug_text.0 = text;
}

fn setup_goal_cell(
    mut commands: Commands,
    mut graph: ResMut<CellGraph>,
    mut goal_cell_id: ResMut<GoalCellId>,
    mut room_map: ResMut<RoomMap>,
) {
    use rand::prelude::*;

    let cell_size = graph.cell_size();
    let horizontal_directions = [Direction::North, Direction::South, Direction::East, Direction::West];

    let candidates: Vec<(CellId, Vec3, Direction)> = graph.cells()
        .flat_map(|cell| {
            let id = cell.id();
            let pos = cell.position();
            cell.boundary_faces()
                .into_iter()
                .filter(|d| horizontal_directions.contains(d))
                .map(move |d| (id, pos, d))
                .collect::<Vec<_>>()
        })
        .collect();

    if candidates.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let (parent_id, parent_pos, direction) = *candidates.choose(&mut rng).unwrap();

    let new_pos = parent_pos + direction.to_vec3() * cell_size;
    let goal_id = graph.add_cell(new_pos);
    graph.connect_cells(parent_id, direction, goal_id);

    goal_cell_id.0 = Some(goal_id);

    // Create an exit room for the goal cell and connect it to the parent's room
    let exit_room_id = room_map.create_room(Color::srgb(1.0, 1.0, 1.0));
    room_map.assign_cell(goal_id, exit_room_id);
    if let Some(parent_room_id) = room_map.get_cell_room(parent_id) {
        room_map.connect_rooms(parent_room_id, parent_id, direction, exit_room_id, goal_id);
    }

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 500_000.0,
            range: 10.0,
            ..default()
        },
        Transform::from_xyz(new_pos.x, new_pos.y + cell_size / 2.0, new_pos.z),
    ));
}

fn check_win_condition(
    player_query: Query<&CellTransform, With<Player>>,
    goal_cell_id: Res<GoalCellId>,
    mut win_state: ResMut<WinState>,
    mut win_text_query: Query<&mut Visibility, With<WinText>>,
) {
    if win_state.0 {
        return;
    }

    let Some(goal_id) = goal_cell_id.0 else { return; };
    let Ok(cell_tf) = player_query.single() else { return; };

    if cell_tf.cell == goal_id {
        win_state.0 = true;
        if let Ok(mut visibility) = win_text_query.single_mut() {
            *visibility = Visibility::Visible;
        }
    }
}
