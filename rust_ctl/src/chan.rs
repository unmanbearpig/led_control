use serde_derive::{Serialize, Deserialize};
use crate::cuboid::{Cuboid};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChanConfig {
    pub index: u16,
    pub min: f64,
    pub max: f64,
    pub exp: f64,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>
}

impl Default for ChanConfig {
    fn default() -> Self {
        ChanConfig {
            index: 0,
            min: 0.0,
            max: 1.0,
            exp: 2.2,
            tags: Vec::new(),
            cuboid: None,
        }
    }
}
