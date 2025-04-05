use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn distance_to(&self, other: &Position) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}
