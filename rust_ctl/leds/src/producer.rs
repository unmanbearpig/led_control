use crate::configuration::Configuration;
use crate::task::TaskMsg;
use std::sync::mpsc;

pub trait Producer<'a>: std::fmt::Debug {
    fn run(&self, config: &Configuration, stop: mpsc::Receiver<TaskMsg>)
        -> Result<(), String>;
}
