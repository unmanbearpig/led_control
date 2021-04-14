
use crate::proto::{Msg, ChanVal, Val};
use crate::msg_handler::MsgHandler;
use std::time;
use std::thread::sleep;
use std::process;
use std::sync::{Arc, Mutex};

struct DemoChan {
    start: f64,
    end: f64,
}

pub fn run<D: MsgHandler + ?Sized>(srv: Arc<Mutex<D>>) -> Result<(), String> {
    println!("running fade...");

    let dur_secs = 3600.0;

    let mut msg: Msg = {
        let srv = srv.lock().map_err(|e| format!("read lock: {:?}", e))?;
        Msg {
            seq_num: 0,
            timestamp: time::SystemTime::now(),
            vals: srv.chans().into_iter()
                .map(|(id, _)| (ChanVal(id, Val::F32(0.0))))
                .collect(),
        }
    };

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(msg.vals.len());

    for _ in 0..msg.vals.len() {
        dchans.push(DemoChan {
            start: 0.0,
            end: 1.0,
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
                // msg.vals[i].1 = Val::F32( ( d.start + (progress.powf(d.exp) * (d.end - d.start)) ) as f32 )
                msg.vals[i].1 = Val::F32( ( d.start + (progress * (d.end - d.start)) ) as f32 )
            }

            {
                let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
                srv.handle_msg(&msg).expect("demo: handle_msg error");
            }

            println!("done");
            process::exit(0);
        }

        for (i, d) in dchans.iter_mut().enumerate() {
            msg.vals[i].1 = Val::F32( ( d.start + (progress * (d.end - d.start)) ) as f32 )
        }

        {
            let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
            srv.handle_msg(&msg).expect("demo: handle_msg error");
        }
        sleep(delay);
    }
}
