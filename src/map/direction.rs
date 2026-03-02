use bevy::math::{Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North, // -Z (Bevy forward)
    South, // +Z (Bevy backward)
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

    /// Converts direction to a unit vector
    pub fn to_vec3(self) -> Vec3 {
        match self {
            Direction::North => Vec3::new(0.0, 0.0, -1.0),
            Direction::South => Vec3::new(0.0, 0.0, 1.0),
            Direction::East => Vec3::new(1.0, 0.0, 0.0),
            Direction::West => Vec3::new(-1.0, 0.0, 0.0),
            Direction::Up => Vec3::new(0.0, 1.0, 0.0),
            Direction::Down => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    /// Returns the Y-axis rotation quaternion for an entity facing this direction.
    /// Assumes Bevy's default entity forward of -Z.
    pub fn to_quat(self) -> Quat {
        match self {
            Direction::North => Quat::IDENTITY,
            Direction::South => Quat::from_rotation_y(std::f32::consts::PI),
            Direction::East  => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            Direction::West  => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Direction::Up | Direction::Down => Quat::IDENTITY,
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

    /// Returns the cardinal direction 90° clockwise when viewed from above.
    pub fn turn_right(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East  => Direction::South,
            Direction::South => Direction::West,
            Direction::West  => Direction::North,
            _                => self,
        }
    }

    /// Returns the cardinal direction 90° counter-clockwise when viewed from above.
    pub fn turn_left(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::West  => Direction::South,
            Direction::South => Direction::East,
            Direction::East  => Direction::North,
            _                => self,
        }
    }
}
