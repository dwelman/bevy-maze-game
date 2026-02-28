use bevy::prelude::*;
use bevy::ui::{JustifyContent, Node, PositionType, Val};
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
    Collider, CellTransform, LerpMovement, LerpRotation, InputRepeatTimer, MovementState,
};
use map::{CellGraph, Direction, spawn_cell_walls};

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
        .insert_resource(DoorMessage {
            text: "Find the door and click on it to win!".to_string(),
            timer: 0.0,
        })
        .insert_resource(DebugVisible(false))
        .add_systems(Startup, (setup_corridor, setup, spawn_cell_walls, spawn_door).chain())
        .add_systems(Update, (handle_player_input, update_cell_movement, update_lerp_rotation))
        .add_systems(Update, (toggle_camera_look_mode, handle_camera_look, update_camera_look_lerp))
        .add_systems(Update, (check_door_click, update_door_message, update_door_message_ui))
        .add_systems(Update, (toggle_debug_text, update_debug_text))
        .run();
}

#[derive(Component)]
struct DebugText;

#[derive(Component)]
struct DoorMessageText;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Door {
    found: bool,
}

#[derive(Resource)]
struct DoorMessage {
    text: String,
    timer: f32,
}

#[derive(Resource)]
struct DebugVisible(bool);

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfig>,
    graph: Res<CellGraph>,
) {
    let creature_height = config.player.creature_height;
    let eye_height = config.player.eye_height;

    // Find the starting cell (cell at position 0,0,0) and get its floor height
    let starting_cell = graph.cells()
        .find(|cell| {
            let pos = cell.position();
            pos.x == 0.0 && pos.z == 0.0
        })
        .expect("Starting cell not found");

    let floor_height = graph.get_cell_floor_height(starting_cell.id()).unwrap();
    let player_y = floor_height + creature_height / 2.0;

    // Player entity (parent) - handles movement, rotation, and collision
    let player = commands.spawn((
        Player,
        Transform::from_xyz(0.0, player_y, 0.0)
            .looking_at(Vec3::new(1.0, player_y, 0.0), Vec3::Y),
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: Vec3::new(0.0, player_y, 0.0),
            start_position: Vec3::new(0.0, player_y, 0.0),
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
        CellTransform {
            cell: starting_cell.id(),
            facing: Direction::East,
        },
    )).id();

    // Camera entity (child) - positioned at eye height, inherits parent rotation
    // eye_height is from feet, player transform is at center, so adjust accordingly
    let camera_y_relative = eye_height - creature_height / 2.0;
    commands.spawn((
        Camera3d::default(),
        CameraLook {
            yaw: 0.0,
            pitch: 0.0,
            target_yaw: 0.0,
            target_pitch: 0.0,
        },
        Transform::from_xyz(0.0, camera_y_relative, 0.0),
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
        Visibility::Hidden,
    ));

    // Door message/instructions UI - top center
    commands.spawn((
        Text::new("Find the door and click on it to win!"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        DoorMessageText,
    ));
}

fn setup_corridor(
    mut commands: Commands,
    mut graph: ResMut<CellGraph>,
) {
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

fn toggle_debug_text(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_visible: ResMut<DebugVisible>,
    mut debug_text_query: Query<&mut Visibility, (With<DebugText>, Without<DoorMessageText>)>,
    mut door_message_query: Query<&mut Visibility, (With<DoorMessageText>, Without<DebugText>)>,
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

        if let Ok(mut visibility) = door_message_query.single_mut() {
            *visibility = if debug_visible.0 {
                Visibility::Hidden
            } else {
                Visibility::Visible
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

fn spawn_door(
    mut commands: Commands,
    graph: Res<CellGraph>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use rand::prelude::*;

    const DOOR_THICKNESS: f32 = 0.2;
    const DOOR_WIDTH: f32 = 0.8;
    const DOOR_HEIGHT: f32 = 1.8;

    let cell_size = graph.cell_size();
    let mut rng = rand::rng();

    // Collect all boundary walls (excluding Up and Down)
    let horizontal_directions = [Direction::North, Direction::South, Direction::East, Direction::West];
    let mut boundary_walls = Vec::new();

    for cell in graph.cells() {
        let cell_position = cell.position();
        let cell_id = cell.id();

        for direction in cell.boundary_faces() {
            // Only consider horizontal walls (not ceiling/floor)
            if horizontal_directions.contains(&direction) {
                boundary_walls.push((cell_id, cell_position, direction));
            }
        }
    }

    if boundary_walls.is_empty() {
        return;
    }

    // Pick a random wall
    let (cell_id, cell_position, direction) = boundary_walls.choose(&mut rng).unwrap();

    // Get the floor height from the cell graph
    let floor_height = graph.get_cell_floor_height(*cell_id).unwrap();
    let door_center_height = floor_height + DOOR_HEIGHT / 2.0;

    // Calculate door size and base position based on direction
    // Position door flush with wall but extending inward into the cell
    let half_cell = cell_size / 2.0;
    let (door_size, wall_offset, lateral_axis) = match direction {
        Direction::North => {
            let size = Vec3::new(DOOR_WIDTH, DOOR_HEIGHT, DOOR_THICKNESS);
            let offset = Vec3::new(0.0, door_center_height, half_cell - DOOR_THICKNESS / 2.0);
            (size, offset, 'x')
        }
        Direction::South => {
            let size = Vec3::new(DOOR_WIDTH, DOOR_HEIGHT, DOOR_THICKNESS);
            let offset = Vec3::new(0.0, door_center_height, -half_cell + DOOR_THICKNESS / 2.0);
            (size, offset, 'x')
        }
        Direction::East => {
            let size = Vec3::new(DOOR_THICKNESS, DOOR_HEIGHT, DOOR_WIDTH);
            let offset = Vec3::new(half_cell - DOOR_THICKNESS / 2.0, door_center_height, 0.0);
            (size, offset, 'z')
        }
        Direction::West => {
            let size = Vec3::new(DOOR_THICKNESS, DOOR_HEIGHT, DOOR_WIDTH);
            let offset = Vec3::new(-half_cell + DOOR_THICKNESS / 2.0, door_center_height, 0.0);
            (size, offset, 'z')
        }
        _ => return, // Skip Up/Down
    };

    // Calculate random lateral offset (aligned to grid)
    // For a 5-unit wall with 0.8-unit door, we can place it at different positions
    let max_offset = ((cell_size - DOOR_WIDTH) / 2.0).floor() as i32;
    let lateral_offset = rng.random_range(-max_offset..=max_offset) as f32;

    let lateral_vec = match lateral_axis {
        'x' => Vec3::new(lateral_offset, 0.0, 0.0),
        'z' => Vec3::new(0.0, 0.0, lateral_offset),
        _ => Vec3::ZERO,
    };

    let door_position = *cell_position + wall_offset + lateral_vec;

    // Create door material (different color from walls)
    let door_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2), // Brown color for the door
        perceptual_roughness: 0.8,
        ..default()
    });

    // Spawn the door
    let mesh = meshes.add(Cuboid::new(door_size.x, door_size.y, door_size.z));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(door_material),
        Transform::from_translation(door_position),
        Collider { size: door_size },
        Door { found: false },
    ));
}

fn check_door_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<&Transform, With<Player>>,
    mut door_query: Query<(&Transform, &Collider, &mut Door)>,
    mut door_message: ResMut<DoorMessage>,
    config: Res<GameConfig>,
) {
    // Check if left mouse button was just pressed
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok((door_transform, door_collider, mut door)) = door_query.single_mut() else {
        return;
    };

    // Skip if already found
    if door.found {
        return;
    }

    // Create ray from camera through cursor position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Check if ray intersects with door's AABB
    let door_pos = door_transform.translation;
    let door_min = door_pos - door_collider.size * 0.5;
    let door_max = door_pos + door_collider.size * 0.5;

    if ray_intersects_aabb(ray.origin, *ray.direction, door_min, door_max) {
        // Check if player is within grid_unit distance
        let player_pos = player_transform.translation;
        let distance = player_pos.distance(door_pos);

        if distance <= config.player.grid_unit {
            door.found = true;
            door_message.text = "You found the door!".to_string();
            door_message.timer = 5.0; // Show for 5 seconds
        } else {
            door_message.text = "Move closer to the door!".to_string();
            door_message.timer = 3.0; // Show for 3 seconds
        }
    }
}

/// Ray-AABB intersection test
fn ray_intersects_aabb(origin: Vec3, direction: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> bool {
    let dir_inv = Vec3::new(
        1.0 / direction.x,
        1.0 / direction.y,
        1.0 / direction.z,
    );

    let t1 = (aabb_min.x - origin.x) * dir_inv.x;
    let t2 = (aabb_max.x - origin.x) * dir_inv.x;
    let t3 = (aabb_min.y - origin.y) * dir_inv.y;
    let t4 = (aabb_max.y - origin.y) * dir_inv.y;
    let t5 = (aabb_min.z - origin.z) * dir_inv.z;
    let t6 = (aabb_max.z - origin.z) * dir_inv.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    // If tmax < 0, ray is intersecting AABB but the whole AABB is behind us
    if tmax < 0.0 {
        return false;
    }

    // If tmin > tmax, ray doesn't intersect AABB
    if tmin > tmax {
        return false;
    }

    true
}

fn update_door_message(
    mut door_message: ResMut<DoorMessage>,
    time: Res<Time>,
) {
    if door_message.timer > 0.0 {
        door_message.timer -= time.delta_secs();
        if door_message.timer <= 0.0 {
            door_message.text = "Find the door and click on it to win!".to_string();
        }
    }
}

fn update_door_message_ui(
    door_message: Res<DoorMessage>,
    config: Res<GameConfig>,
    mut message_text_query: Query<&mut Text, With<DoorMessageText>>,
) {
    let Ok(mut text) = message_text_query.single_mut() else {
        return;
    };

    // Build the full message with controls
    let message_with_controls = format!(
        "{}\n\nCONTROLS: {} {} {} {} - Move | {} {} - Turn | {} - Look | Click - Interact",
        door_message.text,
        config.controls.move_forward,
        config.controls.move_backward,
        config.controls.move_left,
        config.controls.move_right,
        config.controls.rotate_left,
        config.controls.rotate_right,
        config.controls.look_hold,
    );

    text.0 = message_with_controls;
}
