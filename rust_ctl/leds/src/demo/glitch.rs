use crate::msg_handler::MsgHandler;
use proto::v1::{ChanVal, Msg, Val};
use rand::{self, Rng};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time;

use crate::action::Action;
use crate::config::Config;
use crate::srv::Srv;

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Glitch;
impl Action<'_> for Glitch {
    fn perform(&self, config: &Config) -> Result<(), String> {
        run(Srv::init_from_config(&config.configuration)?)
    }
}

struct DemoChan {
    freq: f64,
    min: f64,
    max: f64,
    phi: f64,
}

pub fn run<D: MsgHandler + ?Sized>(srv: Arc<Mutex<D>>) -> Result<(), String> {
    println!("running glitch demo...");

    let mut msg: Msg = {
        let srv = srv.clone();
        let srv = srv.lock().map_err(|e| format!("{:?}", e))?;

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

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(msg.vals.len());

    let freq_dist = rand::distributions::Uniform::new(0.1, 0.3);
    let mut rng = rand::thread_rng();
    for _ in 0..msg.vals.len() {
        dchans.push(DemoChan {
            freq: rng.sample(freq_dist),
            min: 0.1,
            max: 1.0,
            phi: 0.0,
        });
    }

    let delay = time::Duration::from_millis(20);
    let mut t = time::Instant::now();

    loop {
        let dt = t.elapsed().as_secs_f64();
        t = time::Instant::now();
        for (i, d) in dchans.iter_mut().enumerate() {
            let amp = d.max - d.min;
            let delta = dt * d.freq * std::f64::consts::PI * 2.0;
            let new_sin = (d.phi.sin() * (amp / 2.0) + amp / 2.0).powf(2.2) * 2.0 + d.min;
            d.phi += delta;
            msg.vals[i].1 = Val::F32(new_sin as f32);
        }

        // for ChanVal(_, v) in msg.vals.iter() {
        //     let v = match v {
        //         Val::F32(v) => v,
        //         _ => unreachable!(),
        //     };
        //     print!("{:02.8} ", v);
        // }
        // println!("");

        {
            let mut srv = srv.lock().map_err(|e| format!("{:?}", e))?;
            srv.handle_msg(&msg).expect("demo: handle_msg error");
        }
        sleep(delay);
    }
}
