
use crate::srv;
use crate::proto::{Msg, ChanVal, Val};
use std::time;
use std::thread::sleep;
use std::process;

struct DemoChan {
    start: f64,
    end: f64,
    exp: f64,
}

pub fn run(srv: &mut srv::Srv) -> Result<(), String> {
    println!("running fade...");

    let dur_secs = 5.0;

    let mut msg: Msg = Msg {
        seq_num: 0,
        timestamp: time::SystemTime::now(),
        vals: srv.chans().iter()
            .map(|(id, _)| (ChanVal(*id, Val::F32(0.0))))
            .collect(),
    };

    if msg.vals.len() < 3 {
        return Err(format!("at least 3 channels required, got {}", msg.vals.len()));
    }

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(msg.vals.len());

    // temporarily assume first 3 chans are red blue green
    // todo: change these to be set in config
    dchans.push(DemoChan {
        start: 0.0,
        end: 1.0,
        exp: 2.4,
    });

    dchans.push(DemoChan {
        start: 0.0,
        end: 1.0,
        exp: 2.2,
    });

    dchans.push(DemoChan {
        start: 0.0,
        end: 1.0,
        exp: 3.2,
    });

    for _ in 0..(msg.vals.len() -3) {
        dchans.push(DemoChan {
            start: 0.0,
            end: 1.0,
            exp: 2.2,
        });
    }


    let delay = time::Duration::from_millis(10);
    let start_t = time::Instant::now();

    loop {
        let elapsed = start_t.elapsed().as_secs_f64();
        let progress = elapsed / dur_secs;
        if progress > 1.0 {
            let progress: f64 = 1.0;

            for (i, d) in dchans.iter_mut().enumerate() {
                msg.vals[i].1 = Val::F32( ( d.start + (progress.powf(d.exp) * (d.end - d.start)) ) as f32 )
            }
            srv.handle_msg(&msg).expect("demo: handle_msg error");

            println!("done");
            process::exit(0);
        }

        for (i, d) in dchans.iter_mut().enumerate() {
            msg.vals[i].1 = Val::F32( ( d.start + (progress.powf(d.exp) * (d.end - d.start)) ) as f32 )
        }

        srv.handle_msg(&msg).expect("demo: handle_msg error");
        sleep(delay);
    }
}
