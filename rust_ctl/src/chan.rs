use crate::cuboid::Cuboid;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChanConfig {
    pub index: u16,
    pub min: f64,
    pub max: f64,
    pub exp: f64,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>,
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

impl ChanConfig {
    pub fn adjust_value(&self, val: f32) -> f32 {
        (
            self.min +
                (val as f64).powf(self.exp)
                * (self.dynamic_range())
        ) as f32
    }

    pub fn dynamic_range(&self) -> f64 {
        self.max - self.min
    }

    pub fn unadjust_value(&self, val: f32) -> f32 {
        (((val as f64) - self.min) / (self.dynamic_range()))
            .powf(1.0 / self.exp) as f32
    }
}

mod tests {
    extern crate test;
    #[allow(unused_imports)]
    use super::ChanConfig;

    #[test]
    fn test_adjust_chan_val_reverse() {
        let chan_config = ChanConfig {
            index: 0, min: 0.15, max: 0.86, exp: 2.2,
            tags: Vec::new(), cuboid: None,
        };

        assert_eq!(0.4,
                   chan_config.unadjust_value(
                       chan_config.adjust_value(0.4)));
    }
}
