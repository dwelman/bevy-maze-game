#![allow(dead_code)]
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East  => Direction::West,
            Direction::West  => Direction::East,
            Direction::Up    => Direction::Down,
            Direction::Down  => Direction::Up,
        }
    }

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

    /// Returns the coordinate delta for this direction.
    /// Convention: North = -Z, South = +Z, East = +X, West = -X, Up = +Y, Down = -Y
    /// This matches Bevy's default camera facing (-Z is forward).
    pub fn to_ivec3(self) -> IVec3 {
        match self {
            Direction::North => IVec3::new( 0,  0, -1),
            Direction::South => IVec3::new( 0,  0,  1),
            Direction::East  => IVec3::new( 1,  0,  0),
            Direction::West  => IVec3::new(-1,  0,  0),
            Direction::Up    => IVec3::new( 0,  1,  0),
            Direction::Down  => IVec3::new( 0, -1,  0),
        }
    }

    #[allow(dead_code)]
    pub fn to_vec3(self) -> Vec3 {
        self.to_ivec3().as_vec3()
    }
}
