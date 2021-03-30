
extern crate tiny_http;

use std::sync::{Arc, RwLock};
use std::sync::mpsc;

use crate::msg_handler::MsgHandler;

struct Web<T: MsgHandler> {
    srv: Arc<RwLock<T>>,
    pub listen_addr: String,
}

const DEFAULT_LISTEN_ADDR: &str = "localhost:7373";

impl<T: MsgHandler> Web<T> {
    pub fn new(srv: Arc<RwLock<T>>, listen_addr: Option<String>) -> Result<Self, String> {
        let listen_addr = listen_addr.unwrap_or(
            DEFAULT_LISTEN_ADDR.to_string());

        Ok(Web {
            srv: srv,
            listen_addr: listen_addr,
        })
    }
}
