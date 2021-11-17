use serde_derive::{Deserialize, Serialize};
use crate::chan_spec::ChanSpec;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    name: String,
    chans: Vec<ChanSpec>,
    value: f32,
}
