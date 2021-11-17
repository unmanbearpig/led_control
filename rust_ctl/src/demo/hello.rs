use crate::frame::Frame;
use crate::dev::{DevWrite};
use crate::task::TaskMsg;
use rand::{self, Rng};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time;
use crate::action::Action;
use crate::config::Config;

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Hello;
impl Action<'_> for Hello {
    fn perform(&self, config: &Config) -> Result<(), String> {
        run(config.init_srv()?)
    }
}

struct DemoChan {
    freq: f64,
    min: f64,
    max: f64,
    phi: f64,
}

pub fn run<T: DevWrite + ?Sized>(dev: Arc<Mutex<T>>) -> Result<(), String> {
    let (_sender, receiver) = mpsc::channel::<TaskMsg>();

    // runs indefinitely
    run_with_channel(dev, receiver)
}

pub fn run_with_channel<T: DevWrite + ?Sized>(
    dev: Arc<Mutex<T>>,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello demo...");

    let num_chans = {
        let dev = dev.lock().map_err(|e| format!("read lock: {:?}", e))?;
        dev.num_chans()
    };
    let mut frame = Frame::new(num_chans);

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(num_chans as usize);

    let freq_dist = rand::distributions::Uniform::new(0.09, 0.5);
    let mut rng = rand::thread_rng();
    for _ in 0..num_chans {
        dchans.push(DemoChan {
            freq: rng.sample(freq_dist),
            min: 0.70,
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
            frame.set(i as u16, new_sin as f32);
        }

        {
            let mut dev = dev.lock().map_err(|e| format!("write lock: {:?}", e))?;
            dev.set_frame(&frame)?;
            dev.sync()?;
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
    }
}
