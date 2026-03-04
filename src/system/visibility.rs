use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::map::wall::{CellDoorway, CellWall};
use crate::map::{CellGraph, CellId, CardinalDirection, RoomMap};
use crate::system::movement::{CellTransform, TargetCell, TargetFacing};

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
        compute_visible_cells(effective_cell, effective_facing, &graph, &room_map, &mut visible.cells);
    } else {
        // Idle — clear and recompute from scratch so stale cells are pruned.
        visible.cells.clear();
        compute_visible_cells(effective_cell, effective_facing, &graph, &room_map, &mut visible.cells);
    }

    if let Some(player_room) = room_map.get_cell_room(effective_cell) {
        prune_overlapping_cells(&mut visible.cells, player_room, &graph, &room_map);
    }
}

/// Sets `Visibility::Hidden` or `Visibility::Visible` on every `CellWall` and
/// `CellDoorway` entity depending on whether its cell is in the visible set.
pub fn apply_cell_visibility(
    visible: Res<VisibleCells>,
    mut wall_query: Query<(&CellWall, &mut Visibility)>,
    mut doorway_query: Query<(&CellDoorway, &mut Visibility), Without<CellWall>>,
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
}

/// Populates `visible` with every CellId the player can see from `origin`
/// facing `facing`.
///
/// Casts three rays (forward, left, right) and expands one cell in each
/// perpendicular direction at every cell along each ray.
fn compute_visible_cells(
    origin: CellId,
    facing: CardinalDirection,
    graph: &CellGraph,
    room_map: &RoomMap,
    visible: &mut HashSet<CellId>,
) {
    visible.insert(origin);

    let forward = facing;
    let left = facing.turn_left();
    let right = facing.turn_right();
    let backward = facing.opposite();

    cast_ray_with_perpendiculars(origin, forward, backward, graph, room_map, visible);
    cast_ray_with_perpendiculars(origin, left, backward, graph, room_map, visible);
    cast_ray_with_perpendiculars(origin, right, backward, graph, room_map, visible);
}

/// Walks from `origin` in `ray_direction` through cell connections until
/// hitting a wall. For every cell on the ray (including the origin), adds
/// the cell and casts a full perpendicular ray in each perpendicular direction.
fn cast_ray_with_perpendiculars(
    origin: CellId,
    ray_direction: CardinalDirection,
    backward: CardinalDirection,
    graph: &CellGraph,
    room_map: &RoomMap,
    visible: &mut HashSet<CellId>,
) {
    let perp_left = ray_direction.turn_left();
    let perp_right = ray_direction.turn_right();

    // Expand perpendiculars from the origin cell for this ray,
    // but never cast a perpendicular ray in the backward direction.
    if perp_left != backward {
        cast_ray(origin, perp_left, graph, room_map, visible);
    }
    if perp_right != backward {
        cast_ray(origin, perp_right, graph, room_map, visible);
    }

    // Walk the ray.
    let mut current = origin;
    loop {
        let Some(cell) = graph.get_cell(current) else {
            break;
        };
        let Some(next_id) = cell.get_neighbor(ray_direction) else {
            break;
        };

        visible.insert(next_id);

        // The cell immediately past a doorway acts as a 1-cell aperture:
        // perpendicular expansion is suppressed there so you can't see
        // sideways through a narrow opening.  This only applies when the
        // main ray continues further — if the ray ends at this cell there
        // is nothing to "look through", so perpendiculars open up normally.
        let crossed_doorway = room_map.get_cell_room(current) != room_map.get_cell_room(next_id);
        let ray_continues = graph.get_cell(next_id)
            .and_then(|c| c.get_neighbor(ray_direction))
            .is_some();
        let suppress_perps = crossed_doorway && ray_continues;
        if !suppress_perps {
            if perp_left != backward {
                cast_ray(next_id, perp_left, graph, room_map, visible);
            }
            if perp_right != backward {
                cast_ray(next_id, perp_right, graph, room_map, visible);
            }
        }

        current = next_id;
    }
}

/// Walks a straight line from `origin` in `direction` until hitting a wall
/// or a doorway (room boundary), marking each cell as visible.
/// Does not expand perpendiculars.
fn cast_ray(
    origin: CellId,
    direction: CardinalDirection,
    graph: &CellGraph,
    room_map: &RoomMap,
    visible: &mut HashSet<CellId>,
) {
    let mut current = origin;
    loop {
        let Some(cell) = graph.get_cell(current) else {
            break;
        };
        let Some(next_id) = cell.get_neighbor(direction) else {
            break;
        };

        visible.insert(next_id);

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
    // Group visible cells by quantised position.
    // Positions are exact multiples of cell_size so rounding is safe.
    let mut by_position: HashMap<[i32; 3], Vec<CellId>> = HashMap::new();
    for &cell_id in visible.iter() {
        if let Some(cell) = graph.get_cell(cell_id) {
            let p = cell.position();
            let key = [
                (p.x * 1000.0).round() as i32,
                (p.y * 1000.0).round() as i32,
                (p.z * 1000.0).round() as i32,
            ];
            by_position.entry(key).or_default().push(cell_id);
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
    use crate::map::CellGraph;
    use bevy::math::Vec3;

    /// Helper: compute visibility and return the HashSet.
    fn visible_from(
        graph: &CellGraph,
        origin: CellId,
        facing: CardinalDirection,
    ) -> HashSet<CellId> {
        let room_map = RoomMap::new();
        let mut visible = HashSet::new();
        compute_visible_cells(origin, facing, graph, &room_map, &mut visible);
        visible
    }

    #[test]
    fn isolated_cell_sees_only_self() {
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::ZERO);
        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert_eq!(vis, HashSet::from([c0]));
    }

    #[test]
    fn forward_ray_sees_straight_corridor() {
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(0.0, 0.0, -3.0));
        let c2 = graph.add_cell(Vec3::new(0.0, 0.0, -6.0));
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
        let c_behind2 = graph.add_cell(Vec3::new(0.0, 0.0, 6.0));
        let c_behind = graph.add_cell(Vec3::new(0.0, 0.0, 3.0));
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(0.0, 0.0, -3.0));
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
    fn perpendicular_rays_extend_fully() {
        //       c_left
        //         |
        // c0 -- c1 -- c2 (forward = East)
        //         |
        //       c_right
        //         |
        //       c_far_right
        //
        // Perpendicular rays from c1 now walk the full line,
        // so c_far_right IS visible (straight South from c1).
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(3.0, 0.0, 0.0));
        let c2 = graph.add_cell(Vec3::new(6.0, 0.0, 0.0));
        let c_left = graph.add_cell(Vec3::new(3.0, 0.0, -3.0));
        let c_right = graph.add_cell(Vec3::new(3.0, 0.0, 3.0));
        let c_far_right = graph.add_cell(Vec3::new(3.0, 0.0, 6.0));

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
            vis.contains(&c_far_right),
            "perpendicular ray should extend fully, not just 1 cell"
        );
    }

    #[test]
    fn left_and_right_rays_visible() {
        //    c_left -- c0 -- c_right
        // Player at c0 facing North.
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c_left = graph.add_cell(Vec3::new(-3.0, 0.0, 0.0));
        let c_right = graph.add_cell(Vec3::new(3.0, 0.0, 0.0));
        graph.connect_cells(c0, CardinalDirection::West, c_left);
        graph.connect_cells(c0, CardinalDirection::East, c_right);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c_left));
        assert!(vis.contains(&c_right));
    }

    #[test]
    fn left_ray_perpendicular_skips_backward() {
        // Player at c0 facing North. Left ray goes West.
        // Perpendiculars of the West ray are North and South.
        // South is the backward direction, so it's filtered out.
        //
        //       c_left_north
        //           |
        //  c_left -- c0     (player faces North)
        //           |
        //       c_left_south  (NOT visible — backward perp filtered)
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c_left = graph.add_cell(Vec3::new(-3.0, 0.0, 0.0));
        let c_left_north = graph.add_cell(Vec3::new(-3.0, 0.0, -3.0));
        let c_left_south = graph.add_cell(Vec3::new(-3.0, 0.0, 3.0));

        graph.connect_cells(c0, CardinalDirection::West, c_left);
        graph.connect_cells(c_left, CardinalDirection::North, c_left_north);
        graph.connect_cells(c_left, CardinalDirection::South, c_left_south);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c_left_north), "forward perp from left ray should be visible");
        assert!(!vis.contains(&c_left_south), "backward perp from left ray should be filtered");
    }

    #[test]
    fn perpendicular_ray_sees_side_corridor() {
        // L-shaped corridor: c0 --East--> c1 --North--> c2 --North--> c3
        // Player at c0 facing East. The perpendicular ray from c1 goes North,
        // so c2 and c3 are both visible (full perpendicular ray).
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(3.0, 0.0, 0.0));
        let c2 = graph.add_cell(Vec3::new(3.0, 0.0, -3.0));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        assert!(vis.contains(&c2));

        let c3 = graph.add_cell(Vec3::new(3.0, 0.0, -6.0));
        graph.connect_cells(c2, CardinalDirection::North, c3);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(
            vis.contains(&c3),
            "perpendicular ray should see full side corridor"
        );
    }

    #[test]
    fn perpendicular_does_not_recurse_further() {
        // c0 --East--> c1 --North(perp)--> c2 --East--> c3
        // Player at c0 facing East. c2 is on a perpendicular ray from c1.
        // c3 is East of c2, but perpendicular rays don't spawn their own
        // perpendiculars, so c3 should NOT be visible.
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(3.0, 0.0, 0.0));
        let c2 = graph.add_cell(Vec3::new(3.0, 0.0, -3.0));
        let c3 = graph.add_cell(Vec3::new(6.0, 0.0, -3.0));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);
        graph.connect_cells(c2, CardinalDirection::East, c3);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c2), "c2 is on perpendicular ray");
        assert!(
            !vis.contains(&c3),
            "c3 is off the perpendicular ray — requires a second turn"
        );
    }
}
