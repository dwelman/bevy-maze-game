use bevy::math::{IVec3, Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalDirection {
    North,  // -Z (Bevy forward)
    South,  // +Z (Bevy backward)
    East,   // +X
    West,   // -X
    Zenith, // +Y
    Nadir,  // -Y
}

impl CardinalDirection {
    /// Returns the opposite direction
    pub fn opposite(self) -> CardinalDirection {
        match self {
            CardinalDirection::North => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::North,
            CardinalDirection::East => CardinalDirection::West,
            CardinalDirection::West => CardinalDirection::East,
            CardinalDirection::Zenith => CardinalDirection::Nadir,
            CardinalDirection::Nadir => CardinalDirection::Zenith,
        }
    }

    /// Converts direction to a unit vector
    pub fn to_vec3(self) -> Vec3 {
        match self {
            CardinalDirection::North => Vec3::new(0.0, 0.0, -1.0),
            CardinalDirection::South => Vec3::new(0.0, 0.0, 1.0),
            CardinalDirection::East => Vec3::new(1.0, 0.0, 0.0),
            CardinalDirection::West => Vec3::new(-1.0, 0.0, 0.0),
            CardinalDirection::Zenith => Vec3::new(0.0, 1.0, 0.0),
            CardinalDirection::Nadir => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    /// Converts direction to an integer unit vector (grid step)
    pub fn to_ivec3(self) -> IVec3 {
        match self {
            CardinalDirection::North => IVec3::new(0, 0, -1),
            CardinalDirection::South => IVec3::new(0, 0, 1),
            CardinalDirection::East => IVec3::new(1, 0, 0),
            CardinalDirection::West => IVec3::new(-1, 0, 0),
            CardinalDirection::Zenith => IVec3::new(0, 1, 0),
            CardinalDirection::Nadir => IVec3::new(0, -1, 0),
        }
    }

    /// Returns the Y-axis rotation quaternion for an entity facing this direction.
    /// Assumes Bevy's default entity forward of -Z.
    pub fn to_quat(self) -> Quat {
        match self {
            CardinalDirection::North => Quat::IDENTITY,
            CardinalDirection::South => Quat::from_rotation_y(std::f32::consts::PI),
            CardinalDirection::East  => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            CardinalDirection::West  => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            CardinalDirection::Zenith | CardinalDirection::Nadir => Quat::IDENTITY,
        }
    }

    /// Returns all six directions
    pub fn all() -> [CardinalDirection; 6] {
        [
            CardinalDirection::North,
            CardinalDirection::South,
            CardinalDirection::East,
            CardinalDirection::West,
            CardinalDirection::Zenith,
            CardinalDirection::Nadir,
        ]
    }

    /// Returns the cardinal direction 90° clockwise when viewed from above.
    pub fn turn_right(self) -> CardinalDirection {
        match self {
            CardinalDirection::North => CardinalDirection::East,
            CardinalDirection::East  => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::West,
            CardinalDirection::West  => CardinalDirection::North,
            _                        => self,
        }
    }

    /// Returns the cardinal direction 90° counter-clockwise when viewed from above.
    pub fn turn_left(self) -> CardinalDirection {
        match self {
            CardinalDirection::North => CardinalDirection::West,
            CardinalDirection::West  => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::East,
            CardinalDirection::East  => CardinalDirection::North,
            _                        => self,
        }
    }
}
