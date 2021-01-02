
use crate::srv;
use crate::proto::{Msg, ChanVal, Val};
use std::time;
use std::thread::sleep;
use rand::{self, Rng};

struct DemoChan {
    freq: f64,
    min: f64,
    max: f64,
    phi: f64,
}

pub fn run(srv: &mut srv::Srv) -> Result<(), String> {
    println!("running hello demo...");

    let mut msg: Msg = Msg {
        seq_num: 0,
        timestamp: time::SystemTime::now(),
        vals: srv.chans().iter()
            .map(|(id, _)| (ChanVal(*id, Val::F32(0.0))))
            .collect(),
    };


    let mut dchans: Vec<DemoChan> = Vec::with_capacity(msg.vals.len());

    let freq_dist = rand::distributions::Uniform::new(0.001, 0.01);
    let mut rng = rand::thread_rng();
    for _ in 0..msg.vals.len() {
        dchans.push(DemoChan {
            freq: rng.sample(freq_dist),
            min: 0.7,
            max: 1.0,
            phi: 0.0,
        });
    }

    let delay = time::Duration::from_micros(2000);
    let mut t = time::Instant::now();

    loop {
        let dt = t.elapsed().as_secs_f64();
        t = time::Instant::now();
        for (i, d) in dchans.iter_mut().enumerate() {
            let amp = (d.max - d.min);
            let delta = dt * d.freq * std::f64::consts::PI * 2.0;
            // let new_sin = ( d.phi.sin() * (amp / 2.0) + amp / 2.0).powf(2.2) * 2.2 + d.min;
            let new_sin = ( ((d.phi.sin() + 1.0) / 2.0) * amp) + d.min;
            d.phi += delta;
            msg.vals[i].1 = Val::F32(new_sin as f32);
        }

        dbg!(&msg.vals);

        srv.handle_msg(&msg).expect("demo: handle_msg error");
        sleep(delay);
    }
}
