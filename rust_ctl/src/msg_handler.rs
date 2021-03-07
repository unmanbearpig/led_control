use crate::dev::{Dev};
use crate::proto::{ChanId, Msg};
use crate::cuboid::Cuboid;

#[derive(Clone)]
pub struct ChanDescription {
    pub chan_id: u16,
    pub name: String,
    pub tags: Vec<String>,
    pub cuboid: Option<Cuboid>,
}

pub trait MsgHandler
where Self: Dev + Send + Sync
{
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String>;
    fn chans(&self) -> Vec<(ChanId, String)>;
    fn chan_descriptions(&self) -> Vec<ChanDescription>;
}
