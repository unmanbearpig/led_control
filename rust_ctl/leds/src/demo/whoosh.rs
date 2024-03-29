use crate::msg_handler::MsgHandler;
use proto::v1::{ChanVal, Msg, Val};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{self, Duration};

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Whoosh;

#[derive(Debug)]
struct DemoChan {
    min: f64,
    max: f64,
    position: f64,
}

pub fn run<D: MsgHandler + ?Sized>(srv: Arc<Mutex<D>>) -> Result<(), String> {
    println!("running hello whoosh...");

    let mut msg: Msg = {
        let srv = srv.lock().map_err(|e| format!("read lock: {:?}", e))?;

        Msg {
            seq_num: 0,
            timestamp: time::SystemTime::now(),
            vals: srv
                .chans()
                .into_iter()
                .map(|(id, _)| (ChanVal(id, Val::F32(0.0))))
                .collect(),
        }
    };

    let num_chans = msg.vals.len();
    let mut dchans: Vec<DemoChan> = Vec::with_capacity(num_chans);

    for i in 0..msg.vals.len() {
        dchans.push(DemoChan {
            min: 0.01,
            max: 1.0,
            position: 0.1 + (0.8 * (i as f64 / num_chans as f64)),
        });
    }

    let delay = time::Duration::from_millis(13);
    let mut t = time::Instant::now();
    let radius = 0.5;
    let period = Duration::from_millis(1000);
    let start = -radius;
    let finish = 1.0 + radius;
    let mut loc = start;

    loop {
        let dt = t.elapsed().as_secs_f64();
        t = time::Instant::now();

        let pos_diff = dt / period.as_secs_f64();
        loc += pos_diff;
        if loc > finish {
            loc = (loc - finish) + start;
        }
        // dbg!(loc);

        for (i, d) in dchans.iter_mut().enumerate() {
            // msg.vals[i].1 = Val::F32(new_sin as f32);
            let dist = {
                if d.position > loc {
                    d.position - loc
                } else {
                    loc - d.position
                }
            };
            // let val = d.max * (radius - (1.0 / dist)).max(0.0);

            // let val = d.min + (radius - dist).max(0.0).powf(2.2) * d.max;
            let val = (d.min + (radius - dist).max(0.0) * 1.3 * d.max).min(d.max);

            msg.vals[i].1 = Val::F32(val as f32);
            // dbg!(d.position, loc, dist, val);
        }

        dbg!(&msg.vals);

        {
            let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
            srv.handle_msg(&msg).expect("demo: handle_msg error");
        }
        sleep(delay);
    }
}
