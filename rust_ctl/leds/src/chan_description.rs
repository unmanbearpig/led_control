use crate::proto::{ChanId};
use crate::tag::{self};
use crate::chan::ChanConfig;

#[derive(Clone)]
pub struct ChanDescription {
    pub chan_id: u16,
    pub name: String,
    pub human_description: String,
    pub config: ChanConfig,
}

impl ChanDescription {
    pub fn new(chan_id: u16, name: String, config: ChanConfig) -> Self {
        let tags_str = tag::tags_to_str(&config.tags);
        let human_description =
            format!("{} [ {}]: {}", chan_id, tags_str, name);
        ChanDescription {
            chan_id,
            name,
            human_description,
            config,
        }
    }
}

pub trait HasChanDescriptions {
    fn chans(&self) -> Vec<(ChanId, String)>;
    fn chan_descriptions(&self) -> Vec<ChanDescription>;
    fn chans_with_descriptions(&self) -> Vec<(ChanId, String, ChanDescription)> {
        let chans = self.chans();
        let chan_descriptions = self.chan_descriptions();

        let mut result = Vec::new();
        for ((cid, human_descr), descr) in
                chans.iter().zip(chan_descriptions.iter()) {
            result.push((*cid, human_descr.clone(), descr.clone()));
        }

        result.clone()
    }
}
