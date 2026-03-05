use bevy::prelude::*;
use bevy::math::IVec3;
use std::collections::{HashMap, HashSet};

use crate::map::wall::{CellDoorway, CellWall};
use crate::map::{CellGraph, CellId, CardinalDirection, RoomMap};
use crate::system::movement::{CellTransform, TargetCell, TargetFacing};
use crate::{CellLight, GameConfig};

/// Tracks which cells are currently visible to the player based on
/// topology-driven line-of-sight through the cell graph.
#[derive(Resource, Default)]
pub struct VisibleCells {
    cells: HashSet<CellId>,
}

impl VisibleCells {
    pub fn contains(&self, cell_id: CellId) -> bool {
        self.cells.contains(&cell_id)
    }
}

/// Recomputes the visible cell set when the player's cell or facing direction changes.
///
/// Uses a `Local` cache instead of `Changed<CellTransform>` because the rotation
/// system writes to `CellTransform` every frame (triggering change detection even
/// when the value hasn't actually changed).
pub fn update_visible_cells(
    player_query: Query<(&CellTransform, &TargetCell, &TargetFacing)>,
    graph: Res<CellGraph>,
    room_map: Res<RoomMap>,
    config: Res<GameConfig>,
    mut visible: ResMut<VisibleCells>,
    mut last_state: Local<Option<(CellId, CardinalDirection, bool)>>,
) {
    let Ok((cell_tf, target_cell, target_facing)) = player_query.single() else {
        return;
    };

    let effective_cell = target_cell.0.unwrap_or(cell_tf.cell);
    let effective_facing = target_facing.0.unwrap_or(cell_tf.facing);
    let transitioning = target_cell.0.is_some() || target_facing.0.is_some();
    let current_state = (effective_cell, effective_facing, transitioning);
    if *last_state == Some(current_state) {
        return;
    }

    *last_state = Some(current_state);

    if transitioning {
        // During movement/rotation, keep previously visible cells and add the
        // destination's visibility on top. This prevents cells from flickering
        // out mid-transition (e.g. when moving backwards).
        compute_visible_cells(effective_cell, effective_facing, &graph, &mut visible.cells, config.world.render_depth);
    } else {
        // Idle — clear and recompute from scratch so stale cells are pruned.
        visible.cells.clear();
        compute_visible_cells(effective_cell, effective_facing, &graph, &mut visible.cells, config.world.render_depth);
    }

    if let Some(player_room) = room_map.get_cell_room(effective_cell) {
        prune_overlapping_cells(&mut visible.cells, player_room, &graph, &room_map);
    }
}

/// Sets `Visibility::Hidden` or `Visibility::Visible` on every `CellWall`,
/// `CellDoorway`, and `CellLight` entity depending on whether its cell is in
/// the visible set.
pub fn apply_cell_visibility(
    visible: Res<VisibleCells>,
    mut wall_query: Query<(&CellWall, &mut Visibility)>,
    mut doorway_query: Query<(&CellDoorway, &mut Visibility), Without<CellWall>>,
    mut light_query: Query<(&CellLight, &mut Visibility), (Without<CellWall>, Without<CellDoorway>)>,
) {
    if !visible.is_changed() {
        return;
    }

    for (wall, mut vis) in &mut wall_query {
        *vis = if visible.contains(wall.cell_id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (doorway, mut vis) in &mut doorway_query {
        *vis = if visible.contains(doorway.cell_id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (light, mut vis) in &mut light_query {
        *vis = if visible.contains(light.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}


/// Populates `visible` with every CellId the player can see from `origin`
/// facing `facing`.
///
/// Spawns three rays (forward, left, right). Each ray marches in its
/// direction and, at every step, spawns child rays from the left and right
/// neighbour cells going in the same direction. This recursive expansion
/// naturally handles corners and doorways without special-case logic.
fn compute_visible_cells(
    origin: CellId,
    facing: CardinalDirection,
    graph: &CellGraph,
    visible: &mut HashSet<CellId>,
    render_depth: u32,
) {
    visible.insert(origin);
    cast_ray(origin, facing, render_depth, graph, visible);
    cast_ray(origin, facing.turn_left(), render_depth, graph, visible);
    cast_ray(origin, facing.turn_right(), render_depth, graph, visible);
}

/// Walks from `origin` in `direction` up to `depth` steps. At every new cell,
/// spawns child rays from the left and right neighbours (going in the same
/// `direction`), then continues forward. The child rays recurse with
/// decremented depth, so the visible region fans out naturally.
fn cast_ray(
    origin: CellId,
    direction: CardinalDirection,
    depth: u32,
    graph: &CellGraph,
    visible: &mut HashSet<CellId>,
) {
    let left = direction.turn_left();
    let right = direction.turn_right();

    let mut current = origin;
    let mut remaining = depth;

    loop {
        if remaining == 0 {
            break;
        }
        let Some(cell) = graph.get_cell(current) else {
            break;
        };
        let Some(next_id) = cell.get_neighbor(direction) else {
            break;
        };

        visible.insert(next_id);
        remaining -= 1;

        // Spawn child rays from the left/right neighbours of this cell,
        // each continuing in the same forward direction.
        if let Some(next_cell) = graph.get_cell(next_id) {
            if let Some(left_id) = next_cell.get_neighbor(left) {
                visible.insert(left_id);
                cast_ray(left_id, direction, remaining, graph, visible);
            }
            if let Some(right_id) = next_cell.get_neighbor(right) {
                visible.insert(right_id);
                cast_ray(right_id, direction, remaining, graph, visible);
            }
        }

        current = next_id;
    }
}

/// When multiple visible cells share the same world-space position (non-Euclidean
/// overlaps between rooms), keeps only the cell belonging to the player's current
/// room and removes the others.
fn prune_overlapping_cells(
    visible: &mut HashSet<CellId>,
    player_room: crate::map::RoomId,
    graph: &CellGraph,
    room_map: &RoomMap,
) {
    // Group visible cells by grid position.
    // Cell positions are integer grid coordinates so they key directly.
    let mut by_position: HashMap<IVec3, Vec<CellId>> = HashMap::new();
    for &cell_id in visible.iter() {
        if let Some(cell) = graph.get_cell(cell_id) {
            by_position.entry(cell.position()).or_default().push(cell_id);
        }
    }

    for cells in by_position.values() {
        if cells.len() <= 1 {
            continue;
        }

        let has_player_room_cell = cells.iter().any(|&cid| {
            room_map.get_cell_room(cid) == Some(player_room)
        });

        if has_player_room_cell {
            for &cell_id in cells {
                if room_map.get_cell_room(cell_id) != Some(player_room) {
                    visible.remove(&cell_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{CellGraph, RoomMap};
    use bevy::math::IVec3;
    use bevy::prelude::Color;

    /// Helper: compute visibility and return the HashSet.
    fn visible_from(
        graph: &CellGraph,
        origin: CellId,
        facing: CardinalDirection,
    ) -> HashSet<CellId> {
        let mut visible = HashSet::new();
        compute_visible_cells(origin, facing, graph, &mut visible, 10);
        visible
    }

    #[test]
    fn isolated_cell_sees_only_self() {
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(IVec3::ZERO);
        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert_eq!(vis, HashSet::from([c0]));
    }

    #[test]
    fn forward_ray_sees_straight_corridor() {
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(IVec3::new(0, 0, 0));
        let c1 = graph.add_cell(IVec3::new(0, 0, -1));
        let c2 = graph.add_cell(IVec3::new(0, 0, -2));
        graph.connect_cells(c0, CardinalDirection::North, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        assert!(vis.contains(&c2));
    }

    #[test]
    fn backward_cells_not_visible() {
        // c_behind2 -- c_behind -- c0 -- c1 (North)
        // Player at c0 facing North. Nothing behind is visible:
        // backward perpendiculars from the left/right rays are filtered out.
        let mut graph = CellGraph::new(3.0);
        let c_behind2 = graph.add_cell(IVec3::new(0, 0, 2));
        let c_behind  = graph.add_cell(IVec3::new(0, 0, 1));
        let c0        = graph.add_cell(IVec3::new(0, 0, 0));
        let c1        = graph.add_cell(IVec3::new(0, 0, -1));
        graph.connect_cells(c_behind2, CardinalDirection::North, c_behind);
        graph.connect_cells(c_behind, CardinalDirection::North, c0);
        graph.connect_cells(c0, CardinalDirection::North, c1);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        assert!(!vis.contains(&c_behind), "directly behind should not be visible");
        assert!(!vis.contains(&c_behind2), "2 cells behind should not be visible");
    }

    #[test]
    fn perpendicular_expansion_one_cell_then_forward() {
        //       c_left
        //         |
        // c0 -- c1 -- c2 (forward = East)
        //         |
        //       c_right
        //         |
        //       c_far_right
        //
        // The forward ray reaches c1/c2. At c1 it peeks one cell left (c_left)
        // and one cell right (c_right), then spawns forward-going child rays
        // from those cells. c_far_right is deeper sideways and not directly
        // reachable by a forward-going ray, so it is not visible.
        let mut graph = CellGraph::new(3.0);
        let c0          = graph.add_cell(IVec3::new(0, 0, 0));
        let c1          = graph.add_cell(IVec3::new(1, 0, 0));
        let c2          = graph.add_cell(IVec3::new(2, 0, 0));
        let c_left      = graph.add_cell(IVec3::new(1, 0, -1));
        let c_right     = graph.add_cell(IVec3::new(1, 0, 1));
        let c_far_right = graph.add_cell(IVec3::new(1, 0, 2));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::East, c2);
        graph.connect_cells(c1, CardinalDirection::North, c_left);
        graph.connect_cells(c1, CardinalDirection::South, c_right);
        graph.connect_cells(c_right, CardinalDirection::South, c_far_right);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        assert!(vis.contains(&c2));
        assert!(vis.contains(&c_left));
        assert!(vis.contains(&c_right));
        assert!(
            !vis.contains(&c_far_right),
            "deep sideways corridors are covered by the initial left/right rays, not the forward ray's expansion"
        );
    }

    #[test]
    fn left_and_right_rays_visible() {
        //    c_left -- c0 -- c_right
        // Player at c0 facing North.
        let mut graph = CellGraph::new(3.0);
        let c0      = graph.add_cell(IVec3::new(0, 0, 0));
        let c_left  = graph.add_cell(IVec3::new(-1, 0, 0));
        let c_right = graph.add_cell(IVec3::new(1, 0, 0));
        graph.connect_cells(c0, CardinalDirection::West, c_left);
        graph.connect_cells(c0, CardinalDirection::East, c_right);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c_left));
        assert!(vis.contains(&c_right));
    }

    #[test]
    fn left_ray_expands_both_sides() {
        // Player at c0 facing North. Left ray goes West.
        // At c_left, it expands North and South (no backward filtering).
        //
        //       c_left_north
        //           |
        //  c_left -- c0     (player faces North)
        //           |
        //       c_left_south
        let mut graph = CellGraph::new(3.0);
        let c0           = graph.add_cell(IVec3::new(0, 0, 0));
        let c_left       = graph.add_cell(IVec3::new(-1, 0, 0));
        let c_left_north = graph.add_cell(IVec3::new(-1, 0, -1));
        let c_left_south = graph.add_cell(IVec3::new(-1, 0, 1));

        graph.connect_cells(c0, CardinalDirection::West, c_left);
        graph.connect_cells(c_left, CardinalDirection::North, c_left_north);
        graph.connect_cells(c_left, CardinalDirection::South, c_left_south);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c_left_north), "left ray expansion north");
        assert!(vis.contains(&c_left_south), "left ray expansion south");
    }

    #[test]
    fn side_corridor_one_cell_visible() {
        // L-shaped corridor: c0 --East--> c1 --North--> c2 --North--> c3
        // Player at c0 facing East. The forward ray at c1 peeks one cell
        // North to c2 and spawns a child ray going East from c2.
        // c3 is deeper North and not reached by a forward-going ray.
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(IVec3::new(0, 0, 0));
        let c1 = graph.add_cell(IVec3::new(1, 0, 0));
        let c2 = graph.add_cell(IVec3::new(1, 0, -1));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        assert!(vis.contains(&c2), "one cell sideways from main ray");

        let c3 = graph.add_cell(IVec3::new(1, 0, -2));
        graph.connect_cells(c2, CardinalDirection::North, c3);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(
            !vis.contains(&c3),
            "deeper sideways cells are not reached by forward-going child rays"
        );
    }

    #[test]
    fn child_ray_continues_forward_from_side_cell() {
        // c0 --East--> c1 --North(side)--> c2 --East--> c3
        // Player at c0 facing East. The forward ray at c1 peeks North to c2,
        // then spawns a child ray from c2 going East. That child ray reaches
        // c3 — this is the key behaviour for seeing through doorways.
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(IVec3::new(0, 0, 0));
        let c1 = graph.add_cell(IVec3::new(1, 0, 0));
        let c2 = graph.add_cell(IVec3::new(1, 0, -1));
        let c3 = graph.add_cell(IVec3::new(2, 0, -1));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);
        graph.connect_cells(c2, CardinalDirection::East, c3);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c2), "c2 is one cell sideways from main ray");
        assert!(
            vis.contains(&c3),
            "c3 is reached by a child ray from c2 going East"
        );
    }

    // ── prune_overlapping_cells ────────────────────────────────────────────

    /// Helper: build a minimal CellGraph + RoomMap, insert `cells` into both,
    /// run the pruner, and return the surviving visible set.
    fn run_prune(
        positions: &[IVec3],         // grid position for each cell (index = cell slot)
        room_assignments: &[usize],  // room index for each cell (parallel to positions)
        player_room_idx: usize,      // which room the player is in
    ) -> (Vec<CellId>, HashSet<CellId>) {
        let mut graph = CellGraph::new(3.0);
        let mut room_map = RoomMap::new();

        // Create one room per distinct room index.
        let max_room = room_assignments.iter().copied().max().unwrap_or(0);
        let rooms: Vec<_> = (0..=max_room)
            .map(|i| {
                let t = i as f32 / (max_room + 1) as f32;
                room_map.create_room(Color::srgb(t, 1.0 - t, 0.5))
            })
            .collect();

        let cells: Vec<CellId> = positions
            .iter()
            .zip(room_assignments.iter())
            .map(|(&pos, &room_idx)| {
                let cell_id = graph.add_cell(pos);
                room_map.assign_cell(cell_id, rooms[room_idx]);
                cell_id
            })
            .collect();

        let mut visible: HashSet<CellId> = cells.iter().copied().collect();
        prune_overlapping_cells(&mut visible, rooms[player_room_idx], &graph, &room_map);
        (cells, visible)
    }

    #[test]
    fn prune_keeps_player_room_cell_removes_other() {
        // cell_a (room 0, player) and cell_b (room 1) sit at the same position.
        // After pruning, only cell_a survives.
        let (cells, visible) = run_prune(
            &[IVec3::ZERO, IVec3::ZERO],
            &[0, 1],
            0,
        );
        assert!(visible.contains(&cells[0]), "player-room cell must survive pruning");
        assert!(!visible.contains(&cells[1]), "non-player-room cell at same position must be pruned");
    }

    #[test]
    fn prune_does_not_touch_non_overlapping_cells() {
        // cell_a (room 0, player) and cell_b (room 1) are at *different* positions.
        // Neither should be removed.
        let (cells, visible) = run_prune(
            &[IVec3::new(0, 0, 0), IVec3::new(1, 0, 0)],
            &[0, 1],
            0,
        );
        assert!(visible.contains(&cells[0]), "player-room cell must be kept");
        assert!(visible.contains(&cells[1]), "non-overlapping foreign cell must be kept");
    }

    #[test]
    fn prune_no_player_room_cell_in_cluster_keeps_all() {
        // Two cells share a position, but neither belongs to the player's room.
        // The pruner only removes when there IS a player-room cell in the cluster,
        // so both should be left untouched.
        let (cells, visible) = run_prune(
            &[IVec3::ZERO, IVec3::ZERO],
            &[1, 2],  // rooms 1 and 2; player is in room 0
            0,
        );
        assert!(visible.contains(&cells[0]), "cell in room 1 must not be pruned");
        assert!(visible.contains(&cells[1]), "cell in room 2 must not be pruned");
    }

    #[test]
    fn prune_removes_multiple_non_player_cells_at_same_position() {
        // Three cells share a position: one in the player's room, two others.
        // Both non-player cells must be pruned.
        let (cells, visible) = run_prune(
            &[IVec3::ZERO, IVec3::ZERO, IVec3::ZERO],
            &[0, 1, 2],  // room 0 = player
            0,
        );
        assert!(visible.contains(&cells[0]), "player-room cell must survive");
        assert!(!visible.contains(&cells[1]), "first non-player cell must be pruned");
        assert!(!visible.contains(&cells[2]), "second non-player cell must be pruned");
    }

    #[test]
    fn prune_mixed_overlapping_and_unique_positions() {
        // cell_a (room 0, player) and cell_b (room 1) overlap.
        // cell_c (room 1) is at a unique position and must not be touched.
        let pos_overlap = IVec3::new(2, 0, 0);
        let pos_unique  = IVec3::new(3, 0, 0);
        let (cells, visible) = run_prune(
            &[pos_overlap, pos_overlap, pos_unique],
            &[0, 1, 1],
            0,
        );
        assert!(visible.contains(&cells[0]), "overlapping player-room cell must survive");
        assert!(!visible.contains(&cells[1]), "overlapping non-player cell must be pruned");
        assert!(visible.contains(&cells[2]), "non-overlapping foreign cell must be kept");
    }
}
