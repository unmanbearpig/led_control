use std::sync::{Arc, RwLock};
use std::sync::mpsc;

use crate::msg_handler::MsgHandler;
use crate::task::TaskMsg;

pub trait Runner {
    fn run (
        self_lock: Arc<RwLock<Self>>,
        stop: mpsc::Receiver<TaskMsg>)
        -> Result<(), String>;
}
