use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone)]
struct PlayerConfig {
    grid_unit: f32,
    lerp_speed: f32,
    rotation_lerp_speed: f32,
    input_repeat_delay: f32,
}

#[derive(Deserialize)]
struct GameConfig {
    player: PlayerConfig,
}

impl Resource for GameConfig {}

fn main() {
    // Load config
    let config_path = "config.toml";
    let config_str = fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file: {}", config_path));
    let config: GameConfig = toml::from_str(&config_str)
        .expect("Failed to parse config file");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_player_input, update_lerp_movement, update_lerp_rotation))
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
            Transform::IDENTITY.looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
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
        if keyboard_input.pressed(KeyCode::KeyW) {
            if keyboard_input.just_pressed(KeyCode::KeyW) || input_timer.movement_timer >= repeat_delay {
                movement_delta += forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            if keyboard_input.just_pressed(KeyCode::KeyS) || input_timer.movement_timer >= repeat_delay {
                movement_delta -= forward * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            if keyboard_input.just_pressed(KeyCode::KeyA) || input_timer.movement_timer >= repeat_delay {
                movement_delta -= right * grid_unit;
                movement_triggered = true;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            if keyboard_input.just_pressed(KeyCode::KeyD) || input_timer.movement_timer >= repeat_delay {
                movement_delta += right * grid_unit;
                movement_triggered = true;
            }
        }

        // Check rotation inputs
        if keyboard_input.pressed(KeyCode::KeyQ) {
            if keyboard_input.just_pressed(KeyCode::KeyQ) || input_timer.rotation_timer >= repeat_delay {
                rotation_delta = std::f32::consts::FRAC_PI_2; // 90 degrees
                rotation_triggered = true;
            }
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            if keyboard_input.just_pressed(KeyCode::KeyE) || input_timer.rotation_timer >= repeat_delay {
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
