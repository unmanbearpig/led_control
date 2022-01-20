use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::dev::{DevNumChans, DevRead, DevWrite};
use crate::msg_handler::{MsgHandler};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use crate::proto::{ChanId, ChanVal, Msg, Val};
use crate::runner::Runner;
use crate::task::TaskMsg;
use crate::frame::Frame;

#[derive(Debug)]
pub struct MovingAverage<T> {
    pub transition_period: Duration,
    frame_period: Duration,
    frames: VecDeque<Frame<f32>>,
    output: Arc<Mutex<T>>,
    current_frame: Frame<f32>,
    target_frame: Frame<f32>,
    incomplete_target_frame: Frame<f32>,
    last_msg_recv_time: Instant,
    last_msg_target_time: Instant,
}

impl<T: Send + fmt::Display> fmt::Display for MovingAverage<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = self.output.clone();
        let output = output.lock().unwrap();

        write!(
            f,
            "MA period: {}, buffer_len: {}, output to: {}",
            self.transition_period.as_secs_f32(),
            self.frames.len(),
            output
        )
    }
}

impl<T: HasChanDescriptions> DevNumChans for MovingAverage<T> {
    fn num_chans(&self) -> u16 {
        let dev = self.output.lock().unwrap();
        dev.chans().len() as u16
    }
}
impl<T: HasChanDescriptions + DevRead> DevWrite for MovingAverage<T> {
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        self.incomplete_target_frame.set(chan, val);
        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        self.fetch_vals()?;
        self.target_frame = self.incomplete_target_frame.clone();
        self.incomplete_target_frame.clear();
        Ok(())
    }
}

impl<T: DevRead> MovingAverage<T> {
    fn fetch_vals(&mut self) -> Result<(), String> {
        self.clear_frames();
        {
            let output = self.output.lock().unwrap();
            output.get_to_frame(&mut self.current_frame)?;
        }

        for frame in self.frames.iter_mut() {
            *frame = self.current_frame.clone();
        }

        Ok(())
    }
}

impl<T: HasChanDescriptions> DevRead for MovingAverage<T> {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        // maybe we should get the val from the dev, but not sure
        Ok(self.current_frame.get(chan).unwrap_or(0.0))
    }
}

impl<T: DevRead + DevWrite> MovingAverage<T> {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        let mut output = self.output.lock().unwrap();

        if let Err(e) = output.set_frame(&frame) {
            eprintln!("moving_average output.set_frame err: {:?}", e);
            return Err(e)
        }

        if let Err(e) = output.sync() {
            eprintln!("moving_average output.sync err: {:?}", e);
            return Err(e)
        }

        self.current_frame = frame.clone();

        Ok(())
    }
}

impl<T> MovingAverage<T> {
    fn advance_frame(&mut self) {
        self.frames.push_back(self.target_frame.clone());
        self.frames.pop_front();
    }

    fn avg_frame(&self) -> Frame<f32> {
        Frame::simple_average(&self.frames)
    }

    fn has_reached_target(&self) -> bool {
        self.target_frame.is_subset_of(&self.current_frame)
    }

    fn clear_frames(&mut self) {
        for frame in self.frames.iter_mut() {
            frame.clear();
        }
    }
}

impl<T: DevRead + DevWrite> Runner for MovingAverage<T> {
    fn run(
        self_lock: Arc<Mutex<MovingAverage<T>>>,
        stop: mpsc::Receiver<TaskMsg>,
    ) -> Result<(), String> {
        let frame_period = {
            let self_lock = self_lock.clone();
            let self_lock = self_lock.lock().unwrap();
            self_lock.frame_period
        };

        let mut i = 0usize;
        loop {
            {
                i += 1;
                let mov_avg = self_lock.clone();
                let mut mov_avg = mov_avg.lock().unwrap();
                mov_avg.advance_frame();

                let mut avg_frame = mov_avg.avg_frame();

                if mov_avg.has_reached_target() {
                    return Ok(())
                }

                // it's likely that we can't exactly reach the target
                // because of f32 impreciseness
                if i > mov_avg.frames.len() {
                    let furthest_frame =
                        mov_avg.frames[mov_avg.frames.len() -1].clone();

                    // average frame is the same as the last frame
                    // in the VecDeque which means that we've reached
                    // the closest values to the target
                    // TODO is this value any good?
                    if avg_frame.almost_same_as(&furthest_frame, 0.000001) {
                        avg_frame = mov_avg.target_frame.clone();
                    }
                }

                if let Err(e) = mov_avg.set_frame(&avg_frame) {
                    eprintln!("MA error: {}", e);
                }
            }

            match stop.recv_timeout(frame_period) {
                Ok(msg) => match msg {
                    TaskMsg::Pause => {
                        todo!()
                    },
                    TaskMsg::Stop => {
                        return Ok(())
                    },
                    TaskMsg::Ping => {  }
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

impl<T: fmt::Debug + Send + fmt::Display + HasChanDescriptions> MsgHandler for MovingAverage<T> {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        self.target_frame.merge_msg(msg);
        self.last_msg_recv_time = Instant::now();
        self.last_msg_target_time =
            self.last_msg_recv_time + self.transition_period;
        Ok(())
    }
}

impl<T: HasChanDescriptions> HasChanDescriptions for MovingAverage<T> {
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

impl<T: HasChanDescriptions + fmt::Debug> MovingAverage<T> {
    pub fn new(
        output: Arc<Mutex<T>>,
        frame_period: Duration,
        transition_period: Duration,
    ) -> Self {
        let num_chans: usize = {
            let dev = output.lock().unwrap();
            dev.chans().len()
        };

        let frames_num: usize =
            transition_period.div_duration_f32(frame_period).ceil() as usize;

        let mut frames: VecDeque<Frame<f32>> =
            VecDeque::with_capacity(frames_num);
        for _ in 0..frames_num {
            frames.push_back(Frame::new(num_chans as u16));
        }

        let mut vals = Vec::with_capacity(num_chans);
        for i in 0..num_chans {
            vals.push(ChanVal(ChanId(i as u16), Val::F32(0.0)))
        }

        let target_frame = Frame::new(num_chans as u16);

        let now = Instant::now();
        MovingAverage {
            frame_period,
            transition_period,
            frames,
            output,
            current_frame: target_frame.clone(),
            target_frame,
            incomplete_target_frame: Frame::new(num_chans as u16),
            last_msg_recv_time: now,
            last_msg_target_time: now,
        }
    }
}
