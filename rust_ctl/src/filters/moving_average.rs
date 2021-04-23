use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use crate::dev::{DevNumChans, DevRead, DevWrite};
use crate::msg_handler::{MsgHandler};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use crate::proto::{ChanId, ChanVal, Msg, Val};
use crate::runner::Runner;
use crate::task::TaskMsg;
use crate::frame::Frame;

#[derive(Debug)]
pub struct MovingAverage {
    frame_period: Duration,
    transition_period: Duration,
    frames: VecDeque<Frame<f32>>,
    output: Arc<Mutex<dyn MsgHandler>>,
    current_frame: Frame<f32>,
    target_frame: Frame<f32>,
    incomplete_target_frame: Frame<f32>,
    last_msg_recv_time: Instant,
    last_msg_target_time: Instant,
    msg_buf: Msg,
}

impl fmt::Display for MovingAverage {
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

impl DevNumChans for MovingAverage {
    fn num_chans(&self) -> u16 {
        let dev = self.output.lock().unwrap();
        dev.chans().len() as u16
    }
}
impl DevWrite for MovingAverage {
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        self.incomplete_target_frame.set(chan, val);
        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        println!("MA sync");
        self.target_frame = self.incomplete_target_frame.clone();
        self.incomplete_target_frame.clear();
        Ok(())
    }
}
impl DevRead for MovingAverage {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        // maybe we should get the val from the dev, but not sure
        Ok(self.current_frame.get(chan).unwrap_or(0.0))
    }
}

impl MovingAverage {
    fn advance_frame(&mut self) {
        self.frames.push_back(self.target_frame.clone());
        self.frames.pop_front();
    }

    fn num_chans(&self) -> usize {
        // self.frames[0].num_chans()
        self.frames[0].vals.len()
    }

    fn avg_frame(&self) -> Frame<f32> {
        let num_chans = self.num_chans();
        let mut result: Frame<f32> = Frame::new(num_chans as u16);

        for frame in self.frames.iter() {
            for (i, v) in frame.iter().enumerate() {
                result.add_to_val(i as u16, *v);
                //result.vals[i] += v;
            }
        }

        let num_frames = self.frames.len();
        // for item in result.vals.iter_mut().take(num_chans) {
        //     *item /= num_frames as f32;
        // }

        for item in result.iter_mut() {
            *item /= num_frames as f32;
        }

        result
    }

    fn has_reached_target(&self) -> bool {
        self.current_frame == self.target_frame
    }
}

impl Runner for MovingAverage {
    fn run(
        self_lock: Arc<Mutex<MovingAverage>>,
        stop: mpsc::Receiver<TaskMsg>,
    ) -> Result<(), String> {
        let frame_period = {
            let self_lock = self_lock.clone();
            let self_lock = self_lock.lock().unwrap();
            self_lock.frame_period
        };

        loop {
            {
                let mov_avg = self_lock.clone();
                let mut mov_avg = mov_avg.lock().unwrap();
                mov_avg.advance_frame();

                let avg_frame = mov_avg.avg_frame();

                if !mov_avg.has_reached_target() {
                    avg_frame.to_msg(&mut mov_avg.msg_buf);
                    mov_avg.current_frame = avg_frame;
                    let mut output = mov_avg.output.lock().unwrap();
                    match output.handle_msg(&mov_avg.msg_buf) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("moving_average output.handle_msg err: {:?}", e);
                        }
                    }
                }
            }

            match stop.recv_timeout(frame_period) {
                Ok(msg) => match msg {
                    TaskMsg::Stop => return Ok(()),
                    TaskMsg::Ping => {}
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

impl MsgHandler for MovingAverage {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        println!("moving_average got msg");
        // self.target_msg = msg.clone();
        // self.target_frame = Frame::from_msg(msg);
        self.target_frame.merge_msg(msg);
        self.last_msg_recv_time = Instant::now();
        self.last_msg_target_time = self.last_msg_recv_time + self.transition_period;
        Ok(())
    }
}

impl HasChanDescriptions for MovingAverage {
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

impl MovingAverage {
    pub fn new(
        output: Arc<Mutex<dyn MsgHandler>>,
        frame_period: Duration,
        transition_period: Duration,
    ) -> Self {
        let num_chans: usize = {
            let dev = output.lock().unwrap();
            dev.chans().len()
        };

        let frames_num: usize = transition_period.div_duration_f32(frame_period).ceil() as usize;

        let mut frames: VecDeque<Frame<f32>> = VecDeque::with_capacity(frames_num);
        for _ in 0..frames_num {
            frames.push_back(Frame::new(num_chans as u16));
        }

        let mut vals = Vec::with_capacity(num_chans);
        for i in 0..num_chans {
            vals.push(ChanVal(ChanId(i as u16), Val::F32(0.0)))
        }

        let msg_buf = Msg {
            seq_num: 0,
            timestamp: SystemTime::now(),
            vals,
        };

        let target_frame = Frame::new(num_chans as u16);

        MovingAverage {
            frame_period,
            transition_period,
            frames,
            output,
            current_frame: target_frame.clone(),
            target_frame,
            incomplete_target_frame: Frame::new(num_chans as u16),
            msg_buf,
            last_msg_recv_time: Instant::now(),
            last_msg_target_time: Instant::now(),
        }
    }
}
