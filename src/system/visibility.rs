use bevy::prelude::*;
use std::collections::HashSet;

use crate::map::wall::{CellDoorway, CellWall};
use crate::map::{CellGraph, CellId, CardinalDirection};
use crate::system::movement::CellTransform;

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
    player_query: Query<&CellTransform>,
    graph: Res<CellGraph>,
    mut visible: ResMut<VisibleCells>,
    mut last_state: Local<Option<(CellId, CardinalDirection)>>,
) {
    let Ok(cell_tf) = player_query.single() else {
        return;
    };

    let current_state = (cell_tf.cell, cell_tf.facing);
    if *last_state == Some(current_state) {
        return;
    }

    *last_state = Some(current_state);
    visible.cells.clear();
    compute_visible_cells(cell_tf.cell, cell_tf.facing, &graph, &mut visible.cells);
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
    visible: &mut HashSet<CellId>,
) {
    visible.insert(origin);

    let forward = facing;
    let left = facing.turn_left();
    let right = facing.turn_right();

    cast_ray_with_perpendiculars(origin, forward, graph, visible);
    cast_ray_with_perpendiculars(origin, left, graph, visible);
    cast_ray_with_perpendiculars(origin, right, graph, visible);
}

/// Walks from `origin` in `ray_direction` through cell connections until
/// hitting a wall. For every cell on the ray (including the origin), adds
/// the cell and one neighbor in each perpendicular direction.
fn cast_ray_with_perpendiculars(
    origin: CellId,
    ray_direction: CardinalDirection,
    graph: &CellGraph,
    visible: &mut HashSet<CellId>,
) {
    let perp_left = ray_direction.turn_left();
    let perp_right = ray_direction.turn_right();

    // Expand perpendiculars from the origin cell for this ray.
    add_perpendicular_cell(origin, perp_left, graph, visible);
    add_perpendicular_cell(origin, perp_right, graph, visible);

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
        add_perpendicular_cell(next_id, perp_left, graph, visible);
        add_perpendicular_cell(next_id, perp_right, graph, visible);

        current = next_id;
    }
}

/// If `from_cell` has a neighbor in `direction`, inserts that neighbor into `visible`.
fn add_perpendicular_cell(
    from_cell: CellId,
    direction: CardinalDirection,
    graph: &CellGraph,
    visible: &mut HashSet<CellId>,
) {
    if let Some(cell) = graph.get_cell(from_cell) {
        if let Some(neighbor_id) = cell.get_neighbor(direction) {
            visible.insert(neighbor_id);
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
        let mut visible = HashSet::new();
        compute_visible_cells(origin, facing, graph, &mut visible);
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
    fn no_backward_ray_beyond_one_cell() {
        // c_behind2 -- c_behind -- c0 -- c1 (North)
        // Player at c0 facing North.
        // c_behind is 1 cell behind: visible via perpendicular expansion at origin.
        // c_behind2 is 2 cells behind: NOT visible (no backward ray).
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
        // 1-cell backward peek from perpendicular expansion of left/right rays
        assert!(vis.contains(&c_behind));
        // 2 cells behind: NOT visible
        assert!(!vis.contains(&c_behind2));
    }

    #[test]
    fn perpendicular_expansion_one_deep_only() {
        //       c_left
        //         |
        // c0 -- c1 -- c2 (forward = East)
        //         |
        //       c_right
        //         |
        //       c_far_right  (should NOT be visible)
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
            !vis.contains(&c_far_right),
            "perpendicular must not extend beyond 1 cell"
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
    fn left_ray_perpendiculars_use_ray_direction() {
        // Player at c0 facing North. Left ray goes West.
        // Perpendiculars of the West ray are North and South.
        //
        //       c_left_north
        //           |
        //  c_left -- c0     (player faces North)
        //           |
        //       c_left_south
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c_left = graph.add_cell(Vec3::new(-3.0, 0.0, 0.0));
        let c_left_north = graph.add_cell(Vec3::new(-3.0, 0.0, -3.0));
        let c_left_south = graph.add_cell(Vec3::new(-3.0, 0.0, 3.0));

        graph.connect_cells(c0, CardinalDirection::West, c_left);
        graph.connect_cells(c_left, CardinalDirection::North, c_left_north);
        graph.connect_cells(c_left, CardinalDirection::South, c_left_south);

        let vis = visible_from(&graph, c0, CardinalDirection::North);
        assert!(vis.contains(&c_left_north));
        assert!(vis.contains(&c_left_south));
    }

    #[test]
    fn corner_hides_cells() {
        // L-shaped corridor: c0 --East--> c1 --North--> c2
        // Player at c0 facing East. c2 is around the corner — NOT visible.
        let mut graph = CellGraph::new(3.0);
        let c0 = graph.add_cell(Vec3::new(0.0, 0.0, 0.0));
        let c1 = graph.add_cell(Vec3::new(3.0, 0.0, 0.0));
        let c2 = graph.add_cell(Vec3::new(3.0, 0.0, -3.0));

        graph.connect_cells(c0, CardinalDirection::East, c1);
        graph.connect_cells(c1, CardinalDirection::North, c2);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(vis.contains(&c0));
        assert!(vis.contains(&c1));
        // c2 is perpendicular to the forward ray (North from c1), so it IS
        // visible as a 1-deep perpendicular expansion.
        assert!(vis.contains(&c2));

        // But a cell BEYOND c2 (further north) should not be visible.
        let c3 = graph.add_cell(Vec3::new(3.0, 0.0, -6.0));
        graph.connect_cells(c2, CardinalDirection::North, c3);

        let vis = visible_from(&graph, c0, CardinalDirection::East);
        assert!(
            !vis.contains(&c3),
            "cell beyond the corner should not be visible"
        );
    }
}
