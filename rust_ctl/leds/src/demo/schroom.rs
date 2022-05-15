#![allow(unused)]

use crate::frame::Frame;
use crate::dev::{DevWrite};
use crate::task::TaskMsg;
use rand::{self, Rng};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time;
use crate::action::Action;
use crate::config::Config;
use crate::srv::Srv;

#[derive(Clone, std::fmt::Debug,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Schroom;
// impl Action<'_> for Schroom {
//     fn perform(&self, config: &Config) -> Result<(), String> {
//         run(Srv::init_from_config(&config.configuration)?)
//     }
// }

const DEFAULT_FREQ_MIN: f64 = 0.002;
const DEFAULT_FREQ_MAX: f64 = 0.1;

const DEFAULT_MIN: f64 = 0.40;
const DEFAULT_MAX: f64 = 1.0;

#[derive(Debug, Clone, PartialEq,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct OscConfig {
    freq_min: f64,
    freq_max: f64,
    min: f64,
    max: f64,
    /// raise the value to this power
    adjustment: f64,
}

#[derive(Debug, Clone, PartialEq, Default,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SchroomConfig {
    oscs: Vec<OscConfig>,
}

struct SineVal {
    freq: f64,
    min: f64,
    max: f64,
    adjustment: f64,
    phi: f64,
    blend_mode: BlendMode,
}

impl SineVal {
    fn sine(&mut self, dt_secs: f64) -> f64 {
        let amp = self.max - self.min;
        let new_sin = ((((self.phi.sin() + 1.0) / 2.0) * amp) + self.min)
            .powf(self.adjustment);
        self.phi += dt_secs * self.freq * std::f64::consts::PI * 2.0;
        new_sin
    }
}

enum BlendMode {
    Add,
    Mul,
}


struct DemoChan {
    sines: Vec<SineVal>,
}

impl DemoChan {
    fn sine(&mut self, dt_secs: f64) -> f64 {
        let mut result = 0.0f64;

        for s in self.sines.iter_mut() {
            let sinval = s.sine(dt_secs).min(1.0).max(0.0);
            match s.blend_mode {
                BlendMode::Add => result += sinval,
                BlendMode::Mul => result *= sinval,
            }
        }

        result.min(1.0).max(0.0)
    }
}

pub fn run<T: DevWrite + ?Sized>(dev: Arc<Mutex<T>>) -> Result<(), String> {
    let (_sender, receiver) = mpsc::channel::<TaskMsg>();

    // runs indefinitely
    run_with_channel(dev, receiver)
}

pub fn run_with_config<T: DevWrite + ?Sized>(
    dev: Arc<Mutex<T>>,
    config: SchroomConfig,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello demo...");

    todo!()
    // let num_chans = configs.len();
    // let mut frame = Frame::new(num_chans as u16);

    // let mut dchans: Vec<DemoChan> = Vec::with_capacity(num_chans as usize);

    // let mut rng = rand::thread_rng();
    // // let secondary_freq_dist = rand::distributions::Uniform::new(0.5, 2.0);
    // for chan_conf in configs.iter() {
    //     let freq_dist = rand::distributions::Uniform::new(
    //         chan_conf.freq_min, chan_conf.freq_max);

    //     let main_sine = SineVal {
    //         freq: rng.sample(freq_dist),
    //         min: chan_conf.min,
    //         max: chan_conf.max,
    //         adjustment: chan_conf.adjustment,
    //         phi: 0.0,
    //         blend_mode: BlendMode::Add,
    //     };

    //     let secondary_sine = SineVal {
    //         freq: rng.sample(freq_dist) / 3.1,
    //         min: 0.7,
    //         max: 1.1,
    //         adjustment: 1.0,
    //         phi: 0.0,
    //         blend_mode: BlendMode::Mul,
    //     };

    //     let third_sine = SineVal {
    //         freq: rng.sample(freq_dist) / 7.2,
    //         min: 0.7,
    //         max: 1.1,
    //         adjustment: 1.0,
    //         phi: 0.0,
    //         blend_mode: BlendMode::Mul,
    //     };

    //     dchans.push(DemoChan {
    //         sines: vec![main_sine, secondary_sine, third_sine],
    //     });
    // }

    // let delay = time::Duration::from_micros(10_000);
    // let mut t = time::Instant::now();

    // loop {
    //     let dt = t.elapsed().as_secs_f64();
    //     t = time::Instant::now();
    //     for (i, d) in dchans.iter_mut().enumerate() {
    //         let new_sin = d.sine(dt);
    //         // TODO Probably should be applied earlier, so we stay within
    //         // min-max limits
    //         frame.set(i as u16, new_sin as f32);
    //     }

    //     {
    //         let mut dev = dev.lock().map_err(|e| format!("write lock: {:?}", e))?;
    //         dev.set_frame(&frame)?;
    //         dev.sync()?;
    //     }

    //     match stop.recv_timeout(delay) {
    //         Ok(msg) => {
    //             println!("received task msg: {:?}", msg);
    //             match msg {
    //                 TaskMsg::Pause => {
    //                     todo!()
    //                 },
    //                 TaskMsg::Stop => return Ok(()),
    //                 TaskMsg::Ping => {}
    //             }
    //         }
    //         Err(e) => match e {
    //             mpsc::RecvTimeoutError::Timeout => {}
    //             mpsc::RecvTimeoutError::Disconnected => {
    //                 return Ok(());
    //             }
    //         },
    //     }
    // }
}

pub fn run_with_channel<T: DevWrite + ?Sized>(
    dev: Arc<Mutex<T>>,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello schroom...");

    let config = SchroomConfig::default();

    run_with_config(dev, config, stop)
}
