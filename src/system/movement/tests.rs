use super::*;
    use bevy::ecs::system::SystemState;
    use crate::{PlayerConfig, CameraConfig, DebugConfig};

    fn make_test_config() -> GameConfig {
        GameConfig {
            player: PlayerConfig {
                grid_unit: 1.0,
                lerp_speed: 2.0,
                rotation_lerp_speed: 4.0,
                input_repeat_delay: 0.2,
                collider_size: [1.0, 1.0, 1.0],
            },
            camera: CameraConfig {
                look_mode: crate::LookMode::Relative,
                mouse_sensitivity: 0.5,
                max_look_horizontal: 45.0,
                max_look_up: 45.0,
                max_look_down: 45.0,
                look_lerp_speed: 10.0,
            },
            controls: crate::ControlsConfig {
                move_forward: "W".to_string(),
                move_backward: "S".to_string(),
                move_left: "A".to_string(),
                move_right: "D".to_string(),
                rotate_left: "Q".to_string(),
                rotate_right: "E".to_string(),
                look_hold: "Tab".to_string(),
                look_mode_toggle: "M".to_string(),
            },
            debug: DebugConfig {
                log_level: "info".to_string(),
            },
        }
    }

    fn make_test_time() -> Time {
        Time::<()>::default()
    }

    fn make_test_time_advanced(secs: f32) -> Time {
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(secs));
        time
    }

    fn make_lerp_movement() -> LerpMovement {
        LerpMovement {
            state: MovementState::Idle,
            movement_delta: Vec3::ZERO,
            target_position: Vec3::ZERO,
            start_position: Vec3::ZERO,
            lerp_progress: 1.0,
        }
    }

    #[test]
    fn aabb_intersects_positive_overlap() {
        // Overlapping volumes should register as a collision.
        let hit = aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
        );

        assert!(hit);
    }

    #[test]
    fn aabb_intersects_negative_separated() {
        // Well-separated volumes should not collide.
        let hit = aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        );

        assert!(!hit);
    }

    #[test]
    fn aabb_intersects_edge_touch_is_not_collision() {
        // Touching exactly on a face should not count as a collision.
        let hit = aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
        );

        assert!(!hit);
    }

    #[test]
    fn aabb_intersects_corner_touch_is_not_collision() {
        // Touching only at a corner should not count as a collision.
        let hit = aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(2.0, 0.0, 2.0),
            Vec3::new(2.0, 2.0, 2.0),
        );

        assert!(!hit);
    }

    #[test]
    fn aabb_intersects_corner_overlap_is_collision() {
        // A tiny overlap at the corner should still count as a collision.
        let hit = aabb_intersects(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            Vec3::new(1.99, 0.0, 1.99),
            Vec3::new(2.0, 2.0, 2.0),
        );

        assert!(hit);
    }

    #[test]
    fn snap_to_grid_rounds_to_nearest_cell() {
        // Positions should round to the nearest grid unit.
        let snapped = snap_to_grid(Vec3::new(1.6, 2.0, -2.4), 1.0);

        assert_eq!(snapped, Vec3::new(2.0, 2.0, -2.0));
    }

    #[test]
    fn snap_to_grid_keeps_y_axis() {
        // Y-axis should remain unchanged when snapping to grid.
        let snapped = snap_to_grid(Vec3::new(0.49, 3.25, 0.49), 1.0);

        assert_eq!(snapped, Vec3::new(0.0, 3.25, 0.0));
    }

    #[test]
    fn check_collision_positive_hit() {
        // Overlapping colliders in the world should be detected as a collision.
        let mut world = World::new();
        world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));

        let mut state: SystemState<Query<(&Transform, &Collider), Without<LerpMovement>>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let hit = check_collision(
            Vec3::ZERO,
            Vec3::splat(1.0),
            &query,
        );

        assert!(hit);
    }

    #[test]
    fn check_collision_negative_miss() {
        // Non-overlapping colliders should not be reported as a collision.
        let mut world = World::new();
        world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));

        let mut state: SystemState<Query<(&Transform, &Collider), Without<LerpMovement>>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let hit = check_collision(
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::splat(1.0),
            &query,
        );

        assert!(!hit);
    }

    #[test]
    fn check_collision_ignores_lerp_movement_entities() {
        // Entities with LerpMovement should be filtered out by the query.
        let mut world = World::new();
        world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider {
                size: Vec3::splat(1.0),
            },
            make_lerp_movement(),
        ));

        let mut state: SystemState<Query<(&Transform, &Collider), Without<LerpMovement>>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let hit = check_collision(
            Vec3::ZERO,
            Vec3::splat(1.0),
            &query,
        );

        assert!(!hit);
    }

    #[test]
    fn check_collision_tight_gap_blocks_entity() {
        // A gap narrower than the entity should still report a collision.
        let mut world = World::new();
        world.spawn((
            Transform::from_translation(Vec3::new(-0.6, 0.0, 0.0)),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));
        world.spawn((
            Transform::from_translation(Vec3::new(0.6, 0.0, 0.0)),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));

        let mut state: SystemState<Query<(&Transform, &Collider), Without<LerpMovement>>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let hit = check_collision(Vec3::ZERO, Vec3::splat(0.5), &query);

        assert!(hit);
    }

    #[test]
    fn check_collision_wide_gap_allows_entity() {
        // A gap wider than the entity should allow passage without collision.
        let mut world = World::new();
        world.spawn((
            Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));
        world.spawn((
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            Collider {
                size: Vec3::splat(1.0),
            },
        ));

        let mut state: SystemState<Query<(&Transform, &Collider), Without<LerpMovement>>> =
            SystemState::new(&mut world);
        let query = state.get(&world);

        let hit = check_collision(Vec3::ZERO, Vec3::splat(0.5), &query);

        assert!(!hit);
    }

    #[test]
    fn aabb_intersects_thin_wall_blocks_movement() {
        // Even a thin wall should block movement if it overlaps the entity.
        let wall_pos = Vec3::ZERO;
        let wall_size = Vec3::new(0.1, 2.0, 2.0);
        let entity_pos = Vec3::ZERO;
        let entity_size = Vec3::splat(1.0);

        let hit = aabb_intersects(entity_pos, entity_size, wall_pos, wall_size);

        assert!(hit);
    }

    #[test]
    fn aabb_intersects_thin_wall_clear_when_past_it() {
        // Once fully past a thin wall, there should be no collision.
        let wall_pos = Vec3::ZERO;
        let wall_size = Vec3::new(0.1, 2.0, 2.0);
        let entity_pos = Vec3::new(1.0, 0.0, 0.0);
        let entity_size = Vec3::splat(1.0);

        let hit = aabb_intersects(entity_pos, entity_size, wall_pos, wall_size);

        assert!(!hit);
    }

    #[test]
    fn update_lerp_movement_moves_entity_in_open_space() {
        // Entity should lerp from start position to target over multiple frames.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time());

        let entity = world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider { size: Vec3::splat(1.0) },
            LerpMovement {
                state: MovementState::Idle,
                movement_delta: Vec3::new(1.0, 0.0, 0.0),
                target_position: Vec3::ZERO,
                start_position: Vec3::ZERO,
                lerp_progress: 1.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpMovement, &Collider)>,
            Query<(&Transform, &Collider), Without<LerpMovement>>,
        )> = SystemState::new(&mut world);

        // First update: should initiate movement
        let (time, config, moving_query, collider_query) = update_state.get_mut(&mut world);
        update_lerp_movement(time, config, moving_query, collider_query);
        update_state.apply(&mut world);

        let movement = world.get::<LerpMovement>(entity).unwrap();
        
        assert_eq!(movement.state, MovementState::MovingToTarget);
        assert_eq!(movement.target_position, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn update_lerp_movement_snaps_to_grid_on_start() {
        // Entity at off-grid position should snap to grid when movement starts.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time());

        let entity = world.spawn((
            Transform::from_translation(Vec3::new(0.3, 0.0, 0.7)),
            Collider { size: Vec3::splat(1.0) },
            LerpMovement {
                state: MovementState::Idle,
                movement_delta: Vec3::new(1.0, 0.0, 0.0),
                target_position: Vec3::ZERO,
                start_position: Vec3::ZERO,
                lerp_progress: 1.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpMovement, &Collider)>,
            Query<(&Transform, &Collider), Without<LerpMovement>>,
        )> = SystemState::new(&mut world);

        let (time, config, moving_query, collider_query) = update_state.get_mut(&mut world);
        update_lerp_movement(time, config, moving_query, collider_query);
        update_state.apply(&mut world);

        let movement = world.get::<LerpMovement>(entity).unwrap();
        
        assert_eq!(movement.start_position, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(movement.target_position, Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn update_lerp_movement_blocked_by_collision() {
        // Movement should be cancelled if target position contains a collider.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time());

        // Spawn wall at target position
        world.spawn((
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            Collider { size: Vec3::splat(1.0) },
        ));

        let entity = world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider { size: Vec3::splat(1.0) },
            LerpMovement {
                state: MovementState::Idle,
                movement_delta: Vec3::new(1.0, 0.0, 0.0),
                target_position: Vec3::ZERO,
                start_position: Vec3::ZERO,
                lerp_progress: 1.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpMovement, &Collider)>,
            Query<(&Transform, &Collider), Without<LerpMovement>>,
        )> = SystemState::new(&mut world);

        let (time, config, moving_query, collider_query) = update_state.get_mut(&mut world);
        update_lerp_movement(time, config, moving_query, collider_query);
        update_state.apply(&mut world);

        let transform = world.get::<Transform>(entity).unwrap();
        let movement = world.get::<LerpMovement>(entity).unwrap();
        
        assert_eq!(movement.state, MovementState::Idle);
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(movement.movement_delta, Vec3::ZERO);
    }

    #[test]
    fn update_lerp_movement_diagonal_blocked_by_corner() {
        // Diagonal movement should be blocked if either corner collider is present.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time());

        // Spawn wall blocking forward path (diagonal movement from 0,0 to 1,0,1)
        world.spawn((
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            Collider { size: Vec3::splat(1.0) },
        ));

        let entity = world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider { size: Vec3::splat(1.0) },
            LerpMovement {
                state: MovementState::Idle,
                movement_delta: Vec3::new(1.0, 0.0, 1.0).normalize(),
                target_position: Vec3::ZERO,
                start_position: Vec3::ZERO,
                lerp_progress: 1.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpMovement, &Collider)>,
            Query<(&Transform, &Collider), Without<LerpMovement>>,
        )> = SystemState::new(&mut world);

        let (time, config, moving_query, collider_query) = update_state.get_mut(&mut world);
        update_lerp_movement(time, config, moving_query, collider_query);
        update_state.apply(&mut world);

        let movement = world.get::<LerpMovement>(entity).unwrap();
        
        // Movement should be blocked
        assert_eq!(movement.state, MovementState::Idle);
        assert_eq!(movement.movement_delta, Vec3::ZERO);
    }

    #[test]
    fn update_lerp_movement_completes_at_target() {
        // After sufficient time, entity should reach exact target position and return to idle.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time_advanced(1.0));

        let entity = world.spawn((
            Transform::from_translation(Vec3::ZERO),
            Collider { size: Vec3::splat(1.0) },
            LerpMovement {
                state: MovementState::MovingToTarget,
                movement_delta: Vec3::ZERO,
                target_position: Vec3::new(1.0, 0.0, 0.0),
                start_position: Vec3::ZERO,
                lerp_progress: 0.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpMovement, &Collider)>,
            Query<(&Transform, &Collider), Without<LerpMovement>>,
        )> = SystemState::new(&mut world);

        let (time, config, moving_query, collider_query) = update_state.get_mut(&mut world);
        update_lerp_movement(time, config, moving_query, collider_query);
        update_state.apply(&mut world);

        let transform = world.get::<Transform>(entity).unwrap();
        let movement = world.get::<LerpMovement>(entity).unwrap();
        
        assert_eq!(movement.state, MovementState::Idle);
        assert_eq!(transform.translation, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(movement.lerp_progress, 1.0);
    }

    #[test]
    fn update_lerp_rotation_initiates_from_idle() {
        // Setting rotation_delta when idle should start rotation.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time());

        let entity = world.spawn((
            Transform::from_rotation(Quat::IDENTITY),
            LerpRotation {
                rotation_delta: std::f32::consts::FRAC_PI_2,
                target_rotation: Quat::IDENTITY,
                lerp_progress: 1.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpRotation)>,
        )> = SystemState::new(&mut world);

        let (time, config, rotation_query) = update_state.get_mut(&mut world);
        update_lerp_rotation(time, config, rotation_query);
        update_state.apply(&mut world);

        let rotation = world.get::<LerpRotation>(entity).unwrap();
        
        assert_eq!(rotation.lerp_progress, 0.0);
        assert_eq!(rotation.rotation_delta, 0.0);
        assert_ne!(rotation.target_rotation, Quat::IDENTITY);
    }

    #[test]
    fn update_lerp_rotation_completes_at_target() {
        // After sufficient time, rotation should reach exact target and stop.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time_advanced(1.0));

        let target_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let entity = world.spawn((
            Transform::from_rotation(Quat::IDENTITY),
            LerpRotation {
                rotation_delta: 0.0,
                target_rotation: target_rot,
                lerp_progress: 0.0,
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpRotation)>,
        )> = SystemState::new(&mut world);

        let (time, config, rotation_query) = update_state.get_mut(&mut world);
        update_lerp_rotation(time, config, rotation_query);
        update_state.apply(&mut world);

        let transform = world.get::<Transform>(entity).unwrap();
        let rotation = world.get::<LerpRotation>(entity).unwrap();
        
        assert_eq!(rotation.lerp_progress, 1.0);
        assert_eq!(transform.rotation, target_rot);
    }

    #[test]
    fn update_lerp_rotation_ignores_delta_while_rotating() {
        // rotation_delta should not trigger new rotation until current rotation completes.
        let mut world = World::new();
        world.insert_resource(make_test_config());
        world.insert_resource(make_test_time_advanced(0.1));

        let target_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let entity = world.spawn((
            Transform::from_rotation(Quat::IDENTITY),
            LerpRotation {
                rotation_delta: std::f32::consts::FRAC_PI_2, // This should be ignored
                target_rotation: target_rot,
                lerp_progress: 0.5, // Mid-rotation
            },
        )).id();

        let mut update_state: SystemState<(
            Res<Time>,
            Res<GameConfig>,
            Query<(&mut Transform, &mut LerpRotation)>,
        )> = SystemState::new(&mut world);

        let (time, config, rotation_query) = update_state.get_mut(&mut world);
        update_lerp_rotation(time, config, rotation_query);
        update_state.apply(&mut world);

        let rotation = world.get::<LerpRotation>(entity).unwrap();
        
        // Should still be rotating to the same target, not a new one
        assert!(rotation.lerp_progress > 0.5);
        assert!(rotation.lerp_progress < 1.0);
        assert_eq!(rotation.target_rotation, target_rot);
    }