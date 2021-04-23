use crate::proto::{Msg};
use crate::chan_description::{HasChanDescriptions};

use std::fmt::{Debug, Display};

pub trait MsgHandler
where
    Self: HasChanDescriptions + Display + Debug + Send,
{
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String>;
}
