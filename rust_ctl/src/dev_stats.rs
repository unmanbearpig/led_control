use crate::proto::{ChanId, ChanVal, Val, Msg};
use crate::dev::{Dev};
use crate::msg_handler::{MsgHandler, ChanDescription};

use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::fmt;

#[derive(Clone)]
struct ValStats {
    cnt: u64,
    min: f64,
    max: f64,
    avg: f64,
}

impl Default for ValStats {
    fn default() -> Self {
        ValStats {
            cnt: 0, min: 0.0, max: 0.0, avg: 0.0
        }
    }
}

impl ValStats {
    fn add(&mut self, val: f64) {
        if self.cnt == 0 {
            self.min = val;
            self.max = val;
            self.avg = val;
        }

        if val > self.max {
            self.max = val
        }

        if val < self.min {
            self.min = val
        }

        self.avg = if self.cnt > 0 {
            (self.avg * self.cnt as f64 + val) / (self.cnt + 1) as f64
        } else {
            val
        };

        self.cnt += 1;
    }
}

impl fmt::Display for ValStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cnt: {:7} min: {:3.3}  max: {:3.3}  avg: {:3.3}",
               self.cnt, self.min, self.max, self.avg
        )
    }
}

#[derive(Default)]
struct Stats {
    last_update: Option<Instant>,
    msg_cnt: u64,
    prev_msg_cnt: u64,
    f32_vals: Vec<ValStats>,
    msg_recv_latency_ms: ValStats,
    msg_dups: u64,
    msg_miss: u64,
}

impl Stats {
    fn print(&mut self) {
        let prev_update = self.last_update;
        self.last_update = Some(Instant::now());

        let prev_msg_cnt = self.prev_msg_cnt;
        self.prev_msg_cnt = self.msg_cnt;

        let msg_cnt_since_last_update =
            self.msg_cnt - prev_msg_cnt;

        let msgs_per_sec: f32 = {
            match prev_update {
                Some(t) => {
                    let duration = self.last_update.unwrap() - t;
                    msg_cnt_since_last_update as f32 / duration.as_secs_f32()
                }
                None => 0.0
            }
        };

        let f32_vals = self.f32_vals.clone();
        // self.f32_vals.resize_with(0, Default::default);

        let mut val_str = String::new();
        for stat in f32_vals.iter() {
            val_str += format!("{}\n", stat).as_str();
        }

        println!("total_msgs: {:7}, msgs_per_sec: {:9.4}  latency_ms: {}  dups: {:4}  miss: {:4}  \n{}",
                 self.msg_cnt,
                 msgs_per_sec,
                 format!("{}", self.msg_recv_latency_ms),
                 self.msg_dups,
                 self.msg_miss,
                 val_str,
        );
    }
}

pub struct DevStats<D: MsgHandler> {
    dev: Arc<RwLock<D>>,
    stats: Stats,
    last_msg_seq_num: u16,
}

impl<D: 'static + MsgHandler + Sync> DevStats<D> {
    pub fn new(dev: Arc<RwLock<D>>) -> DevStats<D> {
        DevStats {
            dev: dev.clone(),
            stats: Stats::default(),
            last_msg_seq_num: 0,
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
        self.stats.msg_cnt += 1;

        if msg.seq_num != self.last_msg_seq_num +1 {
            if msg.seq_num <= self.last_msg_seq_num {
                self.stats.msg_dups += 1;
            } else {
                self.stats.msg_miss += 1;
            }
        }

        self.last_msg_seq_num = msg.seq_num;

        let latency = msg.timestamp.elapsed();
        match latency {
            Ok(latency) => {
                self.stats.msg_recv_latency_ms.add(latency.as_secs_f64() / 1000.0);
            }
            Err(e) => {
                println!("msg created time is {} ms in the future", e.duration().as_secs_f64() / 1000.0);
            }
        }


        self.stats.f32_vals.resize_with(self.stats.f32_vals.len().max(msg.vals.len()), Default::default);
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            match val {
                Val::F32(v) => {
                    self.stats.f32_vals[*cid as usize].add(*v as f64)
                },
                _ => {},
            }
        }

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