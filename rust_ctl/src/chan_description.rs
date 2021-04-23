use crate::cuboid::Cuboid;
use crate::proto::{ChanId};

#[derive(Clone)]
pub struct ChanDescription {
    pub chan_id: u16,
    pub name: String,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>,
    pub human_description: String,
}

fn tags_to_str(tags: &[String]) -> String {
    let mut out = String::new();
    for t in tags.iter().rev() {
        out += format!("{} ", t).as_ref();
    }
    out
}

impl ChanDescription {
    pub fn new(chan_id: u16, name: String, tags: Vec<String>, cuboid: Option<Cuboid>) -> Self {
        let tags_str = tags_to_str(&tags);
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
