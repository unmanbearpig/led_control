use std::fmt;

use crate::proto::{ChanId, Msg};
use crate::dev::{Dev};
use crate::msg_handler::{MsgHandler, ChanDescription};

use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use std::sync::atomic::{Ordering, AtomicU64};

#[derive(Default)]
struct Stats {
    num_msgs: AtomicU64,
    prev_num_msgs: AtomicU64,
    last_update: Option<Instant>,
}

impl Stats {
    fn print(&mut self) {
        let prev_update = self.last_update;
        self.last_update = Some(Instant::now());
        let prev_num_msgs = self.prev_num_msgs.load(Ordering::SeqCst);
        self.prev_num_msgs = AtomicU64::new(self.num_msgs.load(Ordering::SeqCst));
        let num_msgs_since_last_update =
            self.num_msgs.load(Ordering::SeqCst) - prev_num_msgs;
        let msgs_per_sec: f32 = {
            match prev_update {
                Some(t) => {
                    let duration = self.last_update.unwrap() - t;
                    num_msgs_since_last_update as f32 / duration.as_secs_f32()
                }
                None => 0.0
            }
        };

        println!("total_msgs: {:7}, msgs_per_sec: {:9.4}",
                 self.num_msgs.load(Ordering::Relaxed),
                 msgs_per_sec);
    }
}

pub struct DevStats<D: MsgHandler> {
    dev: Arc<RwLock<D>>,
    stats: Stats,
}

impl<D: 'static + MsgHandler + Sync> DevStats<D> {
    pub fn new(dev: Arc<RwLock<D>>) -> DevStats<D> {
        DevStats {
            dev: dev.clone(),
            stats: Stats::default(),
        }
    }

}

impl<D: MsgHandler> fmt::Display for DevStats<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dev = self.dev.read().unwrap();
        write!(f, "Stats for {}", dev)
    }
}

impl<D: MsgHandler + Sync> Dev for DevStats<D> {
    fn num_chans(&self) -> u16 {
        let dev = self.dev.read().unwrap();
        dev.num_chans()
    }

    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        let mut dev = self.dev.write().unwrap();
        dev.set_f32(chan, val)
    }

    fn sync(&mut self) -> Result<(), String> {
        let mut dev = self.dev.write().unwrap();
        dev.sync()
    }
}

impl<D: MsgHandler + Sync> MsgHandler for DevStats<D> {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        self.stats.num_msgs.fetch_add(1, Ordering::Relaxed);
        let mut dev = self.dev.write().unwrap();
        dev.handle_msg(msg)
    }

    fn chans(&self) -> Vec<(ChanId, String)> {
        let dev = self.dev.read().unwrap();
        dev.chans()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        let dev = self.dev.read().unwrap();
        dev.chan_descriptions()
    }
}


pub fn start_mon<D: 'static + MsgHandler>(dev: Arc<RwLock<DevStats<D>>>, delay: Duration)
                                -> (JoinHandle<()>, Arc<(Mutex<bool>, Condvar)>) {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    let exiter = Arc::new(Condvar::new());
    let handle = {
        let tpair = pair.clone();
        thread::spawn(move || {
            loop {
                let waiting = tpair.0.lock().unwrap();
                match exiter.wait_timeout(waiting, delay).unwrap().1.timed_out() {
                    false => { // means we got the message
                        return;
                    }
                    _ => {
                        // keep looping
                    }
                }

                let mut dev = dev.write().unwrap();
                dev.stats.print();
            }
        })
    };

    (handle, pair)
}
