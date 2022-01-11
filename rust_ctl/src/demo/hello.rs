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

const DEFAULT_FREQ_MIN: f64 = 0.002;
const DEFAULT_FREQ_MAX: f64 = 0.1;

const DEFAULT_MIN: f64 = 0.40;
const DEFAULT_MAX: f64 = 1.0;


// should derive serialize and deserialize
#[derive(Debug, Clone, PartialEq,
         serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DiscoChanConfig {
    freq_min: f64,
    freq_max: f64,
    min: f64,
    max: f64,
    /// raise the value to this power
    adjustment: f64,
}

impl Default for DiscoChanConfig {
    fn default() -> Self {
        DiscoChanConfig {
            freq_min: DEFAULT_FREQ_MIN,
            freq_max: DEFAULT_FREQ_MAX,
            min: DEFAULT_MIN,
            max: DEFAULT_MAX,
            adjustment: 1.0,
        }
    }
}

struct DemoChan {
    freq: f64,
    min: f64,
    max: f64,
    adjustment: f64,
    phi: f64,
}

pub fn run<T: DevWrite + ?Sized>(dev: Arc<Mutex<T>>) -> Result<(), String> {
    let (_sender, receiver) = mpsc::channel::<TaskMsg>();

    // runs indefinitely
    run_with_channel(dev, receiver)
}

pub fn run_with_config<T: DevWrite + ?Sized>(
    dev: Arc<Mutex<T>>,
    configs: Vec<DiscoChanConfig>,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello demo...");

    let num_chans = configs.len();
    let mut frame = Frame::new(num_chans as u16);

    let mut dchans: Vec<DemoChan> = Vec::with_capacity(num_chans as usize);

    let mut rng = rand::thread_rng();
    for chan_conf in configs.iter() {
        let freq_dist = rand::distributions::Uniform::new(
            chan_conf.freq_min, chan_conf.freq_max);

        dchans.push(DemoChan {
            freq: rng.sample(freq_dist),
            min: chan_conf.min,
            max: chan_conf.max,
            adjustment: chan_conf.adjustment,
            phi: 0.0,
        });
    }

    let delay = time::Duration::from_micros(10_000);
    let mut t = time::Instant::now();

    loop {
        let dt = t.elapsed().as_secs_f64();
        t = time::Instant::now();
        for (i, d) in dchans.iter_mut().enumerate() {
            let amp = d.max - d.min;
            let delta = dt * d.freq * std::f64::consts::PI * 2.0;
            let new_sin = (((d.phi.sin() + 1.0) / 2.0) * amp) + d.min;
            // TODO Probably should be applied earlier, so we stay within
            // min-max limits
            let new_sin = new_sin.powf(d.adjustment);
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

pub fn run_with_channel<T: DevWrite + ?Sized>(
    dev: Arc<Mutex<T>>,
    stop: mpsc::Receiver<TaskMsg>,
) -> Result<(), String> {
    println!("running hello demo...");

    let num_chans = {
        let dev = dev.lock().map_err(|e| format!("read lock: {:?}", e))?;
        dev.num_chans()
    };

    let configs: Vec<DiscoChanConfig> =
        vec![DiscoChanConfig::default(); num_chans as usize];

    run_with_config(dev, configs, stop)
}
