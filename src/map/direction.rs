use bevy::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North, // +Z
    South, // -Z
    East,  // +X
    West,  // -X
    Up,    // +Y
    Down,  // -Y
}

impl Direction {
    /// Returns the opposite direction
    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    /// Converts direction to a unit vector scaled by grid_unit
    pub fn to_vec3(self, grid_unit: f32) -> Vec3 {
        match self {
            Direction::North => Vec3::new(0.0, 0.0, grid_unit),
            Direction::South => Vec3::new(0.0, 0.0, -grid_unit),
            Direction::East => Vec3::new(grid_unit, 0.0, 0.0),
            Direction::West => Vec3::new(-grid_unit, 0.0, 0.0),
            Direction::Up => Vec3::new(0.0, grid_unit, 0.0),
            Direction::Down => Vec3::new(0.0, -grid_unit, 0.0),
        }
    }

    /// Returns all six directions
    pub fn all() -> [Direction; 6] {
        [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::Up,
            Direction::Down,
        ]
    }
}
