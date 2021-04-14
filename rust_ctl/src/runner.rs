use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use crate::task::TaskMsg;

pub trait Runner {
    fn run (
        self_lock: Arc<Mutex<Self>>,
        stop: mpsc::Receiver<TaskMsg>)
        -> Result<(), String>;
}
