use crate::cuboid::Cuboid;
use crate::proto::{ChanId};
use crate::tag::{self, Tag};

#[derive(Clone)]
pub struct ChanDescription {
    pub chan_id: u16,
    pub name: String,
    pub tags: Vec<Tag>,
    pub cuboid: Option<Cuboid>,
    pub human_description: String,
}

impl ChanDescription {
    pub fn new(chan_id: u16, name: String, tags: Vec<Tag>, cuboid: Option<Cuboid>) -> Self {
        let tags_str = tag::tags_to_str(&tags);
        let human_description = format!("{} [ {}]: {}", chan_id, tags_str, name);
        ChanDescription {
            chan_id,
            name,
            tags,
            cuboid,
            human_description,
        }
    }
}

pub trait HasChanDescriptions {
    fn chans(&self) -> Vec<(ChanId, String)>;
    fn chan_descriptions(&self) -> Vec<ChanDescription>;
}
