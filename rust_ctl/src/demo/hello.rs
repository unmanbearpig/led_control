use crate::msg_handler::MsgHandler;
use crate::proto::{ChanVal, Msg, Val};
use crate::task::TaskMsg;
use rand::{self, Rng};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time;

struct DemoChan {
    freq: f64,
    min: f64,
    max: f64,
    phi: f64,
}

pub fn run<T: MsgHandler + ?Sized>(srv: Arc<Mutex<T>>) -> Result<(), String> {
    let (_sender, receiver) = mpsc::channel::<TaskMsg>();

    // runs indefinitely
    run_with_channel(srv, receiver)
}

pub fn run_with_channel<T: MsgHandler + ?Sized>(
    srv: Arc<Mutex<T>>,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello demo...");

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

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(msg.vals.len());

    let freq_dist = rand::distributions::Uniform::new(0.09, 0.5);
    let mut rng = rand::thread_rng();
    for _ in 0..msg.vals.len() {
        dchans.push(DemoChan {
            freq: rng.sample(freq_dist),
            min: 0.78,
            max: 1.0,
            phi: 0.0,
        });
    }

    let delay = time::Duration::from_micros(4000);
    let mut t = time::Instant::now();

    loop {
        let dt = t.elapsed().as_secs_f64();
        t = time::Instant::now();
        for (i, d) in dchans.iter_mut().enumerate() {
            let amp = d.max - d.min;
            let delta = dt * d.freq * std::f64::consts::PI * 2.0;
            let new_sin = (((d.phi.sin() + 1.0) / 2.0) * amp) + d.min;
            d.phi += delta;
            msg.vals[i].1 = Val::F32(new_sin as f32);
        }

        msg.timestamp = time::SystemTime::now();
        msg.seq_num += 1;

        {
            let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
            match srv.handle_msg(&msg) {
                Ok(_) => {}
                Err(e) => {
                    println!("demo: handle_msg error: {:?}", e);
                    continue;
                }
            }
        }

        match stop.recv_timeout(delay) {
            Ok(msg) => {
                println!("received task msg: {:?}", msg);
                match msg {
                    TaskMsg::Stop => return Ok(()),
                    TaskMsg::Ping => {}
                }
            }
            Err(e) => match e {
                mpsc::RecvTimeoutError::Timeout => {}
                mpsc::RecvTimeoutError::Disconnected => {
                    return Ok(());
                }
            },
        }
        // sleep(delay);
    }
}
