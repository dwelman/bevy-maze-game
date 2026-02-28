// use super::*;
//     use bevy::ecs::system::SystemState;
//     use bevy::ecs::system::ParamSet;
//     use crate::{PlayerConfig, CameraConfig, DebugConfig};

//     fn make_test_config() -> GameConfig {
//         GameConfig {
//             player: PlayerConfig {
//                 grid_unit: 1.0,
//                 lerp_speed: 2.0,
//                 rotation_lerp_speed: 4.0,
//                 input_repeat_delay: 0.2,
//                 creature_height: 1.75,
//                 creature_width: 0.6,
//                 eye_height: 1.55,
//             },
//             camera: CameraConfig {
//                 look_mode: crate::LookMode::Relative,
//                 mouse_sensitivity: 0.5,
//                 max_look_horizontal: 45.0,
//                 max_look_up: 45.0,
//                 max_look_down: 45.0,
//                 look_lerp_speed: 10.0,
//             },
//             controls: crate::ControlsConfig {
//                 move_forward: "W".to_string(),
//                 move_backward: "S".to_string(),
//                 move_left: "A".to_string(),
//                 move_right: "D".to_string(),
//                 rotate_left: "Q".to_string(),
//                 rotate_right: "E".to_string(),
//                 look_hold: "Tab".to_string(),
//                 look_mode_toggle: "M".to_string(),
//             },
//             debug: DebugConfig {
//                 log_level: "info".to_string(),
//             },
//         }
//     }

//     fn make_test_time() -> Time {
//         Time::<()>::default()
//     }

//     fn make_test_time_advanced(secs: f32) -> Time {
//         let mut time = Time::<()>::default();
//         time.advance_by(std::time::Duration::from_secs_f32(secs));
//         time
//     }

//     fn make_lerp_movement() -> LerpMovement {
//         LerpMovement {
//             state: MovementState::Idle,
//             movement_delta: Vec3::ZERO,
//             target_position: Vec3::ZERO,
//             start_position: Vec3::ZERO,
//             lerp_progress: 1.0,
//         }
//     }

//     #[test]
//     fn snap_to_grid_rounds_to_nearest_cell() {
//         // Positions should round to the nearest grid unit.
//         let snapped = snap_to_grid(Vec3::new(1.6, 2.0, -2.4), 1.0);

//         assert_eq!(snapped, Vec3::new(2.0, 2.0, -2.0));
//     }

//     #[test]
//     fn snap_to_grid_keeps_y_axis() {
//         // Y-axis should remain unchanged when snapping to grid.
//         let snapped = snap_to_grid(Vec3::new(0.49, 3.25, 0.49), 1.0);

//         assert_eq!(snapped, Vec3::new(0.0, 3.25, 0.0));
//     }

//     #[test]
//     fn check_swept_collision_static_overlap_hit() {
//         // A zero-length sweep (start == end) should detect a direct overlap.
//         let mut world = World::new();
//         let obstacle = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//         )).id();
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::splat(1.0), &query));
//         let _ = obstacle;
//     }

//     #[test]
//     fn check_swept_collision_static_overlap_miss() {
//         // A zero-length sweep far from the obstacle should report no collision.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(!check_swept_collision(mover, Vec3::new(3.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0), Vec3::splat(1.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_ignores_self() {
//         // An entity should never collide with itself.
//         let mut world = World::new();
//         let mover = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//         )).id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         // The only collider in the world is the mover itself — should be no collision.
//         assert!(!check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::splat(1.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_includes_moving_entities() {
//         // Moving entities (those with LerpMovement) must not be ignored.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//             make_lerp_movement(),
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         // The obstacle has LerpMovement but should still block the sweep.
//         assert!(check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::splat(1.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_tight_gap_blocks_entity() {
//         // A gap narrower than the entity should block even a zero-length sweep.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::new(-0.6, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));
//         world.spawn((
//             Transform::from_translation(Vec3::new(0.6, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::splat(0.5), &query));
//     }

//     #[test]
//     fn check_swept_collision_wide_gap_allows_entity() {
//         // A gap wider than the entity should allow a zero-length sweep through.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));
//         world.spawn((
//             Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(!check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::splat(0.5), &query));
//     }

//     #[test]
//     fn check_swept_collision_thin_wall_blocks_sweep() {
//         // A thin wall mid-path should block the sweep even if start and end are clear.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
//             Collider { size: Vec3::new(0.1, 2.0, 2.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(check_swept_collision(mover, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::splat(1.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_thin_wall_clear_when_past_it() {
//         // A sweep that starts fully past a thin wall should not collide.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::new(0.1, 2.0, 2.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         // Moving from x=1 to x=2 — wall at x=0 is behind the mover.
//         assert!(!check_swept_collision(mover, Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::splat(1.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_face_touch_is_not_collision() {
//         // A sweep whose leading face exactly meets an obstacle face should not collide.
//         // Obstacle at x=2, size 2 → left face at x=1. Mover size 2 → expanded left edge at 0.
//         // Sweep from x=-1 to x=0: t_enter = (0 - (-1)) / 1 = 1.0, strict < 1.0 excludes it.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
//             Collider { size: Vec3::new(2.0, 2.0, 2.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         assert!(!check_swept_collision(mover, Vec3::new(-1.0, 0.0, 0.0), Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0), &query));
//     }

//     #[test]
//     fn check_swept_collision_tiny_overlap_is_collision() {
//         // Even a tiny overlap at the end of the sweep should register as a collision.
//         let mut world = World::new();
//         world.spawn((
//             Transform::from_translation(Vec3::new(1.99, 0.0, 1.99)),
//             Collider { size: Vec3::new(2.0, 2.0, 2.0) },
//         ));
//         let mover = world.spawn_empty().id();

//         let mut state: SystemState<Query<(Entity, &Transform, &Collider)>> =
//             SystemState::new(&mut world);
//         let query = state.get(&world);

//         // Zero-length sweep: mover (size 2) at origin overlaps obstacle at (1.99, 0, 1.99).
//         assert!(check_swept_collision(mover, Vec3::ZERO, Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0), &query));
//     }

//     #[test]
//     fn update_lerp_movement_moves_entity_in_open_space() {
//         // Entity should lerp from start position to target over multiple frames.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         let entity = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(1.0, 0.0, 0.0),
//                 target_position: Vec3::ZERO,
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         // First update: should initiate movement
//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let movement = world.get::<LerpMovement>(entity).unwrap();
        
//         assert_eq!(movement.state, MovementState::MovingToTarget);
//         assert_eq!(movement.target_position, Vec3::new(1.0, 0.0, 0.0));
//     }

//     #[test]
//     fn update_lerp_movement_snaps_to_grid_on_start() {
//         // Entity at off-grid position should snap to grid when movement starts.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         let entity = world.spawn((
//             Transform::from_translation(Vec3::new(0.3, 0.0, 0.7)),
//             Collider { size: Vec3::splat(1.0) },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(1.0, 0.0, 0.0),
//                 target_position: Vec3::ZERO,
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let movement = world.get::<LerpMovement>(entity).unwrap();
        
//         assert_eq!(movement.start_position, Vec3::new(0.0, 0.0, 1.0));
//         assert_eq!(movement.target_position, Vec3::new(1.0, 0.0, 1.0));
//     }

//     #[test]
//     fn update_lerp_movement_blocked_by_collision() {
//         // Movement should be cancelled if target position contains a collider.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         // Spawn wall at target position
//         world.spawn((
//             Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));

//         let entity = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(1.0, 0.0, 0.0),
//                 target_position: Vec3::ZERO,
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let transform = world.get::<Transform>(entity).unwrap();
//         let movement = world.get::<LerpMovement>(entity).unwrap();
        
//         assert_eq!(movement.state, MovementState::Idle);
//         assert_eq!(transform.translation, Vec3::ZERO);
//         assert_eq!(movement.movement_delta, Vec3::ZERO);
//     }

//     #[test]
//     fn update_lerp_movement_diagonal_blocked_by_corner() {
//         // Diagonal movement should be blocked if either corner collider is present.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         // Spawn wall blocking forward path (diagonal movement from 0,0 to 1,0,1)
//         world.spawn((
//             Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
//             Collider { size: Vec3::splat(1.0) },
//         ));

//         let entity = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(1.0, 0.0, 1.0).normalize(),
//                 target_position: Vec3::ZERO,
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let movement = world.get::<LerpMovement>(entity).unwrap();
        
//         // Movement should be blocked
//         assert_eq!(movement.state, MovementState::Idle);
//         assert_eq!(movement.movement_delta, Vec3::ZERO);
//     }

//     #[test]
//     fn update_lerp_movement_completes_at_target() {
//         // After sufficient time, entity should reach exact target position and return to idle.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time_advanced(1.0));

//         let entity = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: Vec3::splat(1.0) },
//             LerpMovement {
//                 state: MovementState::MovingToTarget,
//                 movement_delta: Vec3::ZERO,
//                 target_position: Vec3::new(1.0, 0.0, 0.0),
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 0.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let transform = world.get::<Transform>(entity).unwrap();
//         let movement = world.get::<LerpMovement>(entity).unwrap();
        
//         assert_eq!(movement.state, MovementState::Idle);
//         assert_eq!(transform.translation, Vec3::new(1.0, 0.0, 0.0));
//         assert_eq!(movement.lerp_progress, 1.0);
//     }

//     #[test]
//     fn update_lerp_rotation_initiates_from_idle() {
//         // Setting rotation_delta when idle should start rotation.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         let entity = world.spawn((
//             Transform::from_rotation(Quat::IDENTITY),
//             LerpRotation {
//                 rotation_delta: std::f32::consts::FRAC_PI_2,
//                 target_rotation: Quat::IDENTITY,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             Query<(&mut Transform, &mut LerpRotation)>,
//         )> = SystemState::new(&mut world);

//         let (time, config, rotation_query) = update_state.get_mut(&mut world);
//         update_lerp_rotation(time, config, rotation_query);
//         update_state.apply(&mut world);

//         let rotation = world.get::<LerpRotation>(entity).unwrap();
        
//         assert_eq!(rotation.lerp_progress, 0.0);
//         assert_eq!(rotation.rotation_delta, 0.0);
//         assert_ne!(rotation.target_rotation, Quat::IDENTITY);
//     }

//     #[test]
//     fn update_lerp_rotation_completes_at_target() {
//         // After sufficient time, rotation should reach exact target and stop.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time_advanced(1.0));

//         let target_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
//         let entity = world.spawn((
//             Transform::from_rotation(Quat::IDENTITY),
//             LerpRotation {
//                 rotation_delta: 0.0,
//                 target_rotation: target_rot,
//                 lerp_progress: 0.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             Query<(&mut Transform, &mut LerpRotation)>,
//         )> = SystemState::new(&mut world);

//         let (time, config, rotation_query) = update_state.get_mut(&mut world);
//         update_lerp_rotation(time, config, rotation_query);
//         update_state.apply(&mut world);

//         let transform = world.get::<Transform>(entity).unwrap();
//         let rotation = world.get::<LerpRotation>(entity).unwrap();
        
//         assert_eq!(rotation.lerp_progress, 1.0);
//         assert_eq!(transform.rotation, target_rot);
//     }

//     #[test]
//     fn movement_blocked_by_thin_wall_tunnel_through() {
//         // A thin wall (0.1 wide) at x=0.5 should block movement from (0,0,0) to (1,0,0)
//         // even though neither endpoint overlaps the wall.
//         // Player collider: (0.6, 1.75, 0.6), wall: (0.1, 2, 2) at (0.5, 0, 0).
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         // Thin wall that the player would pass through
//         world.spawn((
//             Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
//             Collider {
//                 size: Vec3::new(0.1, 2.0, 2.0),
//             },
//         ));

//         let player_size = Vec3::new(0.6, 1.75, 0.6);
//         let entity = world.spawn((
//             Transform::from_translation(Vec3::ZERO),
//             Collider { size: player_size },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(1.0, 0.0, 0.0),
//                 target_position: Vec3::ZERO,
//                 start_position: Vec3::ZERO,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let transform = world.get::<Transform>(entity).unwrap();
//         let movement = world.get::<LerpMovement>(entity).unwrap();

//         // Movement should be blocked — player must not tunnel through the wall
//         assert_eq!(movement.state, MovementState::Idle, "player should remain idle (blocked)");
//         assert_eq!(transform.translation, Vec3::ZERO, "player should not have moved");
//         assert_eq!(movement.movement_delta, Vec3::ZERO, "movement delta should be consumed");
//     }

//     #[test]
//     fn update_lerp_rotation_ignores_delta_while_rotating() {
//         // rotation_delta should not trigger new rotation until current rotation completes.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time_advanced(0.1));

//         let target_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
//         let entity = world.spawn((
//             Transform::from_rotation(Quat::IDENTITY),
//             LerpRotation {
//                 rotation_delta: std::f32::consts::FRAC_PI_2, // This should be ignored
//                 target_rotation: target_rot,
//                 lerp_progress: 0.5, // Mid-rotation
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             Query<(&mut Transform, &mut LerpRotation)>,
//         )> = SystemState::new(&mut world);

//         let (time, config, rotation_query) = update_state.get_mut(&mut world);
//         update_lerp_rotation(time, config, rotation_query);
//         update_state.apply(&mut world);

//         let rotation = world.get::<LerpRotation>(entity).unwrap();
        
//         // Should still be rotating to the same target, not a new one
//         assert!(rotation.lerp_progress > 0.5);
//         assert!(rotation.lerp_progress < 1.0);
//         assert_eq!(rotation.target_rotation, target_rot);
//     }

//     #[test]
//     fn horizontal_move_not_blocked_by_wall_panel_above_doorway() {
//         // A creature (height=1.75) walking through a 2-cell-tall doorway must not
//         // be blocked by the wall panel belonging to the cell directly above the
//         // opening.
//         //
//         // Grid layout (grid_unit = 1.0, doorway in South wall at z=2):
//         //   Cell (0,0,1) - destination, South face is open (doorway)
//         //   Cell (0,2,1) - above doorway, South face wall panel at z=1.5, y=2.0
//         //
//         // The above-doorway panel's collider spans y=[1.5, 2.5].
//         // Without the Y-axis skip, the Minkowski expansion pulls its lower
//         // boundary down to y=0.625, falsely blocking the creature at y=0.875.
//         //
//         // The creature sweeps from cell (0,0,0) → (0,0,1), i.e. z=0 → z=1.
//         // The panel's Z centre is at z=1.5 (South face of cell z=1), so the
//         // sweep endpoint (z=1) stops short of the panel in XZ — there is no
//         // XZ collision either. The only false collision path was through Y.
//         let mut world = World::new();
//         world.insert_resource(make_test_config());
//         world.insert_resource(make_test_time());

//         // South-face wall panel for the cell above the doorway:
//         // cell y=2, z=1 → panel centred at (0, 2.0, 1.5), size (1.0, 1.0, 0.1).
//         world.spawn((
//             Transform::from_translation(Vec3::new(0.0, 2.0, 1.5)),
//             Collider { size: Vec3::new(1.0, 1.0, 0.1) },
//         ));

//         // Creature height 1.75 — centre sits at y = 1.75/2 = 0.875.
//         let creature_height = 1.75_f32;
//         let start = Vec3::new(0.0, creature_height * 0.5, 0.0);
//         let end   = Vec3::new(0.0, creature_height * 0.5, 1.0);

//         let entity = world.spawn((
//             Transform::from_translation(start),
//             Collider { size: Vec3::new(0.6, creature_height, 0.6) },
//             LerpMovement {
//                 state: MovementState::Idle,
//                 movement_delta: Vec3::new(0.0, 0.0, 1.0),
//                 target_position: start,
//                 start_position: start,
//                 lerp_progress: 1.0,
//             },
//         )).id();

//         let mut update_state: SystemState<(
//             Res<Time>,
//             Res<GameConfig>,
//             ParamSet<(
//                 Query<(Entity, &mut Transform, &mut LerpMovement, &Collider)>,
//                 Query<(Entity, &Transform, &Collider)>,
//             )>,
//         )> = SystemState::new(&mut world);

//         let (time, config, queries) = update_state.get_mut(&mut world);
//         update_lerp_movement(time, config, queries);
//         update_state.apply(&mut world);

//         let movement = world.get::<LerpMovement>(entity).unwrap();

//         // The panel is above the doorway and outside the sweep path — must not block.
//         assert_eq!(movement.state, MovementState::MovingToTarget,
//             "creature should move through the doorway, not be blocked by the panel above it");
//         assert_eq!(movement.target_position, end,
//             "target should be one cell south");
//     }