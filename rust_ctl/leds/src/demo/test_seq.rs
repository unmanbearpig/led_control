use crate::msg_handler::MsgHandler;
use proto::v1::{ChanVal, Msg, Val};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time;

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct TestSeq;

pub fn run<T: MsgHandler + ?Sized>(srv: Arc<Mutex<T>>) -> Result<(), String> {
    println!("running test_seq...");

    let mut msg: Msg = {
        let srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;

        let chans = srv.chans();
        Msg {
            seq_num: 0,
            timestamp: time::SystemTime::now(),
            vals: chans
                .iter()
                .map(|(id, _)| (ChanVal(*id, Val::F32(0.0))))
                .collect(),
        }
    };

    loop {
        for i in 0..msg.vals.len() {
            eprintln!("demo test_seq: chan {}", i);
            let mut fval: f32 = 0.0;
            let min: f32 = 0.0;
            let max: f32 = 1.0;
            let step: f32 = 0.01;
            // lower delay doesn't work for local udp usb server
            // why?
            let delay = time::Duration::from_millis(10);

            let mut set = |fval: f32| {
                msg.vals[i].1 = Val::F32(fval.powf(2.2));
                {
                    let srv = srv.clone();
                    let mut srv = srv.lock().unwrap();
                    srv.handle_msg(&msg).expect("demo: handle_msg error");
                }
                sleep(delay);
            };

            while fval < max {
                fval = (fval + step).min(max);
                set(fval);
            }
            while fval > min {
                fval = (fval - step).max(min);
                set(fval);
            }
        }
    }
}
