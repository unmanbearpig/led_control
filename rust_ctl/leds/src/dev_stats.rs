use crate::msg_handler::{MsgHandler};
use crate::chan_description::{HasChanDescriptions, ChanDescription};
use crate::proto::{ChanId, ChanVal, Msg, Val};
use crate::term_bar;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use crate::frame::Frame;

use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct ValStats {
    cnt: u64,
    min: f64,
    max: f64,
    avg: f64,
    last: f64,
}

impl Default for ValStats {
    fn default() -> Self {
        ValStats {
            cnt: 0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            last: 0.0,
        }
    }
}

impl ValStats {
    fn add(&mut self, val: f64) {
        self.last = val;
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

    fn merge_from(&mut self, from: &ValStats) {
        if from.cnt == 0 {
            return;
        }
        if self.cnt == 0 {
            *self = from.clone();
            return;
        }

        self.min = self.min.min(from.min);
        self.max = self.max.max(from.max);
        self.avg = (self.avg * self.cnt as f64 + from.avg * from.cnt as f64)
            / (self.cnt + from.cnt) as f64;
        self.cnt += from.cnt;
    }
}

impl fmt::Display for ValStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cnt: {:7} min: {:3.3}  avg: {:3.3}  last: {:3.3}  max: {:3.3}",
            self.cnt, self.min, self.avg, self.last, self.max
        )
    }
}

#[derive(Default, Debug)]
struct MsgStats {
    chan_descriptions: Vec<String>,
    last_update: Option<Instant>,
    msg_cnt: u64,
    prev_msg_cnt: u64,
    f32_vals_last: Vec<ValStats>,
    f32_vals_overall: Vec<ValStats>,
    msg_recv_latency_ms: ValStats,
    msg_dups: u64,
    msg_miss: u64,
}

impl MsgStats {
    fn new(chan_descriptions: Vec<ChanDescription>) -> Self {
        let str_descriptions = chan_descriptions
            .iter()
            .map(|cd| cd.human_description.clone())
            .collect();
        MsgStats {
            chan_descriptions: str_descriptions,
            ..Default::default()
        }
    }

    fn merge(into: &mut Vec<ValStats>, from: &[ValStats]) {
        into.resize_with(into.len().max(from.len()), Default::default);

        for (i, v) in from.iter().enumerate() {
            into[i].merge_from(v);
        }
    }

    fn print(&mut self) {
        Self::merge(&mut self.f32_vals_overall, &self.f32_vals_last);

        let prev_update = self.last_update;
        self.last_update = Some(Instant::now());

        let prev_msg_cnt = self.prev_msg_cnt;
        self.prev_msg_cnt = self.msg_cnt;

        let msg_cnt_since_last_update = self.msg_cnt - prev_msg_cnt;

        let msgs_per_sec: f32 = {
            match prev_update {
                Some(t) => {
                    let duration = self.last_update.unwrap() - t;
                    msg_cnt_since_last_update as f32 / duration.as_secs_f32()
                }
                None => 0.0,
            }
        };

        let bar = term_bar::config().len(100);

        let mut val_str = String::new();
        for (i, overall_stat) in self.f32_vals_overall.iter().enumerate() {
            let last_stat = self.f32_vals_last.get(i);

            let (bar_str, last_val_str) = match last_stat {
                Some(last_stat) => (
                    format!("{}", bar.val(last_stat.avg as f32)),
                    format!("{}", last_stat),
                ),
                None => ("".to_string(), "None".to_string()),
            };

            let chan_name = self
                .chan_descriptions
                .get(i)
                .map(|s| s.as_ref())
                .unwrap_or("unnamed channel");
            val_str += format!(
                "\n{}\n{}  {}\n{}\n",
                chan_name, overall_stat, last_val_str, bar_str
            ).as_str();
        }

        self.f32_vals_last.resize_with(0, Default::default);

        println!(
            "msg count: {:7}, msgs per sec: {:9.4}  dups: {:4}  loss: {:4}  \n  latency ms: {}\n{}",
            self.msg_cnt,
            msgs_per_sec,
            self.msg_dups,
            self.msg_miss,
            self.msg_recv_latency_ms,
            val_str,
        );
    }
}

#[derive(Debug)]
pub struct DevWriteStats {
    stats: Vec<ValStats>,
}

impl DevWriteStats {
    fn new() -> Self {
        DevWriteStats {
            stats: Vec::new(),
        }
    }

    // fn frame_complete(&mut self) {
    //     if self.stats.len() < self.incomplete_frame.vals.len() {
    //         self.stats.resize_with(self.incomplete_frame.vals.len(),
    //                                Default::default);
    //     }

    //     for (cid, val) in self.incomplete_frame.iter_some() {
    //         self.stats[cid as usize].add(*val as f64);
    //     }

    //     self.incomplete_frame.clear();
    // }

    fn print(&mut self) {
        for (i, chan) in self.stats.iter().enumerate() {
            println!("Chan {}: {}", i, chan);
        }
    }

    fn has_any_data(&self) -> bool {
        for chan in self.stats.iter() {
            if chan.cnt > 0 {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct DevStats<D> {
    name: String,
    dev: Arc<Mutex<D>>,
    // add chan tags or something
    msg_stats: MsgStats,
    dev_write_stats: DevWriteStats,
    last_msg_seq_num: u16,
}

impl<D: 'static + MsgHandler + Sync> DevStats<D> {
    pub fn new(dev: Arc<Mutex<D>>) -> DevStats<D> {
        let (chan_descriptions, name) = {
            let dev = dev.lock().unwrap();
            let chan_descriptions = dev.chan_descriptions();
            let name = format!("Stats for {}", dev);
            (chan_descriptions, name)
        };

        DevStats {
            name,
            dev,
            msg_stats: MsgStats::new(chan_descriptions),
            dev_write_stats: DevWriteStats::new(),
            last_msg_seq_num: 0,
        }
    }
}

impl<D: MsgHandler + Sync> MsgHandler for DevStats<D> {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        self.msg_stats.msg_cnt += 1;

        if msg.seq_num != self.last_msg_seq_num.overflowing_add(1).0 {
            if msg.seq_num <= self.last_msg_seq_num {
                self.msg_stats.msg_dups += 1;
            } else {
                self.msg_stats.msg_miss += 1;
            }
        }

        self.last_msg_seq_num = msg.seq_num;

        let latency = msg.timestamp.elapsed();
        match latency {
            Ok(latency) => {
                self.msg_stats
                    .msg_recv_latency_ms
                    .add(latency.as_secs_f64() * 1000.0);
            }
            Err(e) => {
                self.msg_stats
                    .msg_recv_latency_ms
                    .add(e.duration().as_secs_f64() * -1000.0);
            }
        }

        self.msg_stats.f32_vals_last.resize_with(
            self.chan_descriptions().len().max(msg.vals.len()),
            Default::default,
        );
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            if let Val::F32(v) = val {
                self.msg_stats.f32_vals_last[*cid as usize].add(*v as f64)
            }
        }

        let mut dev = self.dev.lock().unwrap();
        dev.handle_msg(msg)
    }
}

impl<D: HasChanDescriptions> HasChanDescriptions for DevStats<D> {
    fn chans(&self) -> Vec<(ChanId, String)> {
        let dev = self.dev.lock().unwrap();
        dev.chans()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        let dev = self.dev.lock().unwrap();
        dev.chan_descriptions()
    }
}

impl<D> fmt::Display for DevStats<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<D: DevNumChans> DevNumChans for DevStats<D> {
    fn num_chans(&self) -> u16 {
        let dev = self.dev.lock().unwrap();
        dev.num_chans()
    }
}

impl<D: DevRead> DevRead for DevStats<D> {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        let dev = self.dev.lock().unwrap();
        dev.get_f32(chan)
    }
}

impl<D: DevWrite> DevWrite for DevStats<D> {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        let res = {
            let mut dev = self.dev.lock().unwrap();
            dev.set_frame(frame)
        };
        self.msg_stats.msg_cnt += 1;
        res
    }

    // fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
    //     self.dev_write_stats.incomplete_frame
    //         .set(chan, val);

    //     let mut dev = self.dev.lock().unwrap();
    //     dev.set_f32(chan, val)
    // }

    // fn sync(&mut self) -> Result<(), String> {
    //     let res = {
    //         let mut dev = self.dev.lock().unwrap();
    //         dev.sync()
    //     };
    //     self.msg_stats.msg_cnt += 1;
    //     self.dev_write_stats.frame_complete();
    //     res
    // }
}

impl<D: Dev> Dev for DevStats<D> {
}

pub fn start_mon<D: 'static + Send>(
    dev: Arc<Mutex<DevStats<D>>>,
    delay: Duration,
) -> (JoinHandle<()>, Arc<(Mutex<()>, Condvar)>) {
    let pair = Arc::new((Mutex::new(()), Condvar::new()));

    let exiter = Arc::new(Condvar::new());

    let handle = {
        let tpair = pair.clone();

        thread::spawn(move || {
            loop {
                let waiting = tpair.0.lock().unwrap();
                if !exiter.wait_timeout(waiting, delay).unwrap().1.timed_out() {
                    // means we got the message
                    return;
                }

                let mut dev = dev.lock().unwrap();

                dev.msg_stats.print();
                dev.dev_write_stats.print();

                if dev.dev_write_stats.has_any_data() {
                    dev.dev_write_stats.print();
                }
            }
        })
    };

    (handle, pair)
}
