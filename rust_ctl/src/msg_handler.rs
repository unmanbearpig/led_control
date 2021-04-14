use crate::proto::{ChanId, Msg};
use crate::cuboid::Cuboid;

use std::fmt::{Display, Debug};

#[derive(Clone)]
pub struct ChanDescription {
    pub chan_id: u16,
    pub name: String,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>,
}

impl ChanDescription {
    #[allow(dead_code)]
    pub fn tags_str(&self) -> String {
        let mut out = String::new();
        for t in self.tags.iter() {
            out += format!("{} ", t).as_ref();
        }
        out
    }

    #[allow(dead_code)]
    pub fn humanize(&self) -> String {
        self.tags_str()
    }
}

pub trait MsgHandler
where Self: Display + Debug + Send
{
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String>;
    fn chans(&self) -> Vec<(ChanId, String)>;
    fn chan_descriptions(&self) -> Vec<ChanDescription>;
}
