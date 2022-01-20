use std::sync::mpsc;
use std::fmt;
use crate::dev::{DevNumChans, DevRead, DevWrite};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use crate::proto::{ChanId};
use crate::frame::Frame;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::task::TaskMsg;
use crate::runner::Runner;

pub struct Fade<T> {
    /// We render frames to this
    output: Arc<Mutex<T>>,

    /// The whole duration of the fade from start to finish
    pub fade_duration: Duration,

    /// How long we show each frame, i.e. 1/FPS
    frame_duration: Duration,

    /// When the fade ends
    target_time: Instant,

    /// Fade to this frame
    target_frame: Frame<f32>,

    /// A buffer to store values while doing `set_f32`, before the `sync`.
    /// `sync` clears this and moves to `target_frame`
    incomplete_target_frame: Frame<f32>,

    /// When the fade has started
    start_time: Instant,

    /// Fade from this frame (got from `output` on `sync`)
    from_frame: Frame<f32>,

    /// TODO buffer for rendering
    current_frame: Frame<f32>,
}

impl<T: HasChanDescriptions + fmt::Debug> Fade<T> {
    pub fn new(
        output: Arc<Mutex<T>>,
        frame_duration: Duration,
        fade_duration: Duration,
    ) -> Self {
        let now = Instant::now();
        Fade {
            output: output,
            frame_duration,
            fade_duration,
            target_time: now,
            start_time: now,
            target_frame: Frame::empty(),
            incomplete_target_frame: Frame::empty(),
            from_frame: Frame::empty(),
            current_frame: Frame::empty(),
        }
    }
}

impl<T: HasChanDescriptions> DevNumChans for Fade<T> {
    fn num_chans(&self) -> u16 {
        let dev = self.output.lock().unwrap();
        dev.chans().len() as u16
    }
}

impl<T: DevRead> Fade<T> {
    fn fetch_vals(&mut self) -> Result<(), String> {
        let output = self.output.lock().unwrap();
        output.get_to_frame(&mut self.from_frame)?;

        Ok(())
    }
}


impl<T: HasChanDescriptions + DevRead> DevWrite for Fade<T> {
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        self.incomplete_target_frame.set(chan, val);
        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        self.fetch_vals()?;
        self.start_time = Instant::now();
        self.target_time = self.start_time + self.fade_duration;

        self.target_frame = self.incomplete_target_frame.clone();

        self.incomplete_target_frame.clear();

        self.from_frame.vals.shrink_to(self.target_frame.vals.len());
        self.target_frame.vals.resize(self.from_frame.vals.len(), None);

        self.current_frame.vals.resize(self.target_frame.vals.len(), None);
        Ok(())
    }
}

impl<T: DevRead + DevWrite> Fade<T> {
    fn render_current_frame(&mut self) -> Result<(), String> {
        let mut output = self.output.lock().unwrap();

        if let Err(e) = output.set_frame(&self.current_frame) {
            eprintln!("fade output.set_frame err: {:?}", e);
            return Err(e)
        }

        if let Err(e) = output.sync() {
            eprintln!("fade output.sync err: {:?}", e);
            return Err(e)
        }

        Ok(())
    }
}

impl<T: HasChanDescriptions> HasChanDescriptions for Fade<T> {
    fn chans(&self) -> Vec<(ChanId, String)> {
        let output = self.output.clone();
        let output = output.lock().unwrap();
        output.chans()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        let output = self.output.clone();
        let output = output.lock().unwrap();
        output.chan_descriptions()
    }
}

impl<T: DevRead + DevWrite> Fade<T> {
    fn is_done(&self, t: Instant) -> bool {
        if t >= self.target_time {
            return true;
        }

        if t <= self.start_time {
            panic!("fade haven't reached start time in `run`, \
                   which must be an error");
        }

        false
    }

    fn set_current_frame(&mut self, t: Instant) -> Result<(), String> {
        // At this point `t` is somewhere between `start_time` and
        // `target_time`

        let time_passed: Duration = t.duration_since(self.start_time);
        let mut progress = time_passed.as_secs_f64() /
            self.fade_duration.as_secs_f64();
        if progress > 1.0 {
            progress = 1.0
        }

        for ii in 0..self.target_frame.vals.len() {
            let target_val = self.target_frame.vals[ii];
            let target_val = match target_val {
                Some(val) => val,
                None => continue,
            };
            let from_val = self.from_frame.vals[ii];
            let from_val = match from_val {
                Some(val) => val,
                None => continue,
            };

            let from_val = from_val as f64;
            let target_val = target_val as f64;

            let current_val =
                if target_val > from_val {
                    from_val + (target_val - from_val) * progress
                } else if target_val < from_val {
                    from_val - (from_val - target_val) * progress
                } else {
                    // ==
                    target_val
                };

            self.current_frame.vals[ii] = Some(current_val as f32);
        }

        Ok(())
    }
}

impl<T: DevRead + DevWrite> Runner for Fade<T> {
    fn run(
        self_lock: Arc<Mutex<Fade<T>>>,
        stop: mpsc::Receiver<TaskMsg>,
    ) -> Result<(), String> {
        let frame_duration = {
            let self_lock = self_lock.clone();
            let self_lock = self_lock.lock().unwrap();
            self_lock.frame_duration
        };

        {
            // Make sure frames are the same size
            let fade = self_lock.clone();
            let fade = fade.lock().unwrap();
            assert_eq!(fade.from_frame.vals.len(),
                       fade.target_frame.vals.len());
            assert_eq!(fade.from_frame.vals.len(),
                       fade.current_frame.vals.len());
        }

        // Whether we should set the last value after we reached `target_time`
        // if it's true then we should not, if it's false then we should, and
        // then we should set it to true
        let mut is_paused = false;
        loop {
            {
                let fade = self_lock.clone();
                let mut fade = fade.lock().unwrap();

                let t = Instant::now();
                if fade.is_done(t) {
                    if !is_paused {
                        // set the value the final time, to make sure we've
                        // reached exactly `target_frame`
                        fade.set_current_frame(t)?;
                        fade.render_current_frame()?;
                        is_paused = true;
                    }
                } else {
                    is_paused = false;
                    fade.set_current_frame(t)?;
                    fade.render_current_frame()?;
                }
            }

            // wait for the time to render next frame or a message from outside
            match stop.recv_timeout(frame_duration) {
                Ok(msg) => {
                    println!("fade got msg {:?}", msg);
                    match msg {
                        TaskMsg::Pause => {
                            is_paused = true;
                            let self_lock = self_lock.clone();
                            let mut self_lock = self_lock.lock().unwrap();
                            self_lock.target_time = Instant::now();
                        },
                        TaskMsg::Stop => {
                            return Ok(())
                        },
                        TaskMsg::Ping => {  }
                    }
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Timeout => {}
                    mpsc::RecvTimeoutError::Disconnected => {
                        return Ok(());
                    }
                },
            }
        }
    }
}
