use crate::dev::{DevWrite, DevNumChans};
use crate::frame::Frame;
use proto::v1::{ChanVal, Msg, Val};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time;

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct TestSeq;

pub fn run<T: DevWrite + DevNumChans + ?Sized>(
    srv: Arc<Mutex<T>>) -> Result<(), String> {
    println!("running test_seq...");

    const MIN: f32 = 0.0;
    const MAX: f32 = 1.0;
    const STEP: f32 = 0.01;
    const DELAY: time::Duration = time::Duration::from_millis(10);

    let num_chans = {
        let srv = srv.lock().unwrap();
        srv.num_chans()
    };

    let mut frame = Frame::<f32>::new(num_chans);

    loop {
        for cid in 0..num_chans {
            let set = |frame: &mut Frame<f32>, fval: f32| -> Result<(), String> {
                frame.set(cid, fval);
                {
                    frame.set(cid, fval);
                    let srv = srv.clone();
                    let mut srv = srv.lock().unwrap();
                    srv.set_frame(&frame)?;
                }
                sleep(DELAY);
                Ok(())
            };

            frame.clear();
            let mut fval: f32 = 0.0;

            while fval < MAX {
                fval = (fval + STEP).min(MAX);
                set(&mut frame, fval)?;
            }

            while fval > MIN {
                fval = (fval - STEP).max(MIN);
                set(&mut frame, fval)?;
            }
        }
    }
}
