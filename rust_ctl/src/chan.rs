use crate::demo::hello::DiscoChanConfig;
use crate::cuboid::Cuboid;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChanConfig {
    pub index: u16,
    pub min: f64, // min and max are ignored when exp is None
    pub max: f64,
    pub exp: Option<f64>,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>,
    pub disco_config: Option<DiscoChanConfig>,
}


impl Default for ChanConfig {
    fn default() -> Self {
        ChanConfig {
            index: 0,
            min: 0.0,
            max: 1.0,
            exp: None,
            tags: Vec::new(),
            cuboid: None,
            disco_config: None,
        }
    }
}

impl ChanConfig {
    pub fn dynamic_range(&self) -> f64 {
        self.max - self.min
    }

    pub fn adjust_value(&self, val: f32) -> f32 {
        if let Some(exp) = self.exp {
            (
                self.min +
                    (val as f64).powf(exp)
                    * (self.dynamic_range())
            ) as f32
        } else {
            val
        }
    }

    pub fn unadjust_value(&self, val: f32) -> f32 {
        if let Some(exp) = self.exp {
            (((val as f64) - self.min) / (self.dynamic_range()))
                .powf(1.0 / exp) as f32
        } else {
            val
        }
    }
}

mod tests {
    extern crate test;
    #[allow(unused_imports)]
    use super::ChanConfig;

    #[test]
    fn test_adjust_chan_val_reverse() {
        let chan_config = ChanConfig {
            index: 0, min: 0.15, max: 0.86, exp: Some(2.2),
            tags: Vec::new(), cuboid: None,
        };

        assert_eq!(0.4,
                   chan_config.unadjust_value(
                       chan_config.adjust_value(0.4)));
    }
}
