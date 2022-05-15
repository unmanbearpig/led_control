use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::task::TaskMsg;

pub trait Runner {
    fn run(self_lock: Arc<Mutex<Self>>, stop: mpsc::Receiver<TaskMsg>)
        -> Result<(), String>;
}
