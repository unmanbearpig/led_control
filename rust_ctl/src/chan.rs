use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChanConfig {
    pub index: u16,
    pub min: f64,
    pub max: f64,
    pub exp: f64,
    pub tags: Vec<String>
}

impl Default for ChanConfig {
    fn default() -> Self {
        ChanConfig {
            index: 0,
            min: 0.0,
            max: 1.0,
            exp: 2.2,
            tags: Vec::new(),
        }
    }
}

impl ChanConfig {
    pub fn defaults(num: u16) -> Vec<Self> {
        let mut res = Vec::with_capacity(num as usize);
        for i in  0..num {
            let mut cc = ChanConfig::default();
            cc.index = i;
            res.push(cc);
        }
        res
    }
}
