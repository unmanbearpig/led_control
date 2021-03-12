
use crate::config;
use crate::action;
use crate::chan_spec::{ChanSpec};

use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc;

#[derive(Default)]
struct ControllerState {
    task: Option<Task>,
}

struct Contoller<T: MsgHandler> {
    srv: Arc<RwLock<T>>,
    config: config::Config,
    state: Arc<Mutex<WebState>>,
}

impl<T: MsgHandler> Controller<T> {
}

struct CtlMessage {
    msg: CtlAction,
}

enum CtlAction {
    StopTask,
    FadeTo()
}
