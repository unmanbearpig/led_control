use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Coord {
    pub fn dist_to(&self, other: &Coord) -> f32 {
        (
            (self.x - other.x).powi(2) +
                (self.y - other.y).powi(2) +
                (self.z - other.z).powi(2)
        ).sqrt()
    }
}
