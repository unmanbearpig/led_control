use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use crate::dev::Dev;
use crate::msg_handler::{ChanDescription, MsgHandler};
use crate::proto::{ChanId, ChanVal, Msg, Val};
use crate::runner::Runner;
use crate::task::TaskMsg;

#[derive(Debug, PartialOrd, PartialEq)]
struct Frame {
    vals: Vec<f32>,
}

impl Frame {
    fn new(num_chans: usize) -> Self {
        Frame {
            vals: vec![0.0; num_chans],
        }
    }

    fn from_msg(msg: &Msg) -> Self {
        let mut vals = Vec::with_capacity(msg.vals.len());
        // TODO: should actually use cid
        for ChanVal(ChanId(_cid), val) in msg.vals.iter() {
            match val {
                Val::F32(val) => {
                    vals.push(*val);
                }
                Val::U16(_) => {
                    unimplemented!();
                }
            }
        }

        Frame { vals }
    }

    fn num_chans(&self) -> usize {
        self.vals.len()
    }

    fn to_msg(&self, msg: &mut Msg) {
        for (i, v) in self.vals.iter().enumerate() {
            msg.vals[i].1 = Val::F32(*v)
        }
    }
}

#[derive(Debug)]
pub struct MovingAverage {
    frame_period: Duration,
    transition_period: Duration,
    frames: VecDeque<Frame>,
    output: Arc<Mutex<dyn MsgHandler>>,
    current_msg: Msg,
    target_msg: Msg,
    last_msg_recv_time: Instant,
    last_msg_target_time: Instant,
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

impl Dev for MovingAverage {
    fn num_chans(&self) -> u16 {
        let dev = self.output.lock().unwrap();
        // dev.num_chans()
        dev.chans().len() as u16
    }

    fn set_f32(&mut self, _chan: u16, _val: f32) -> Result<(), String> {
        // let mut dev = self.output.write().unwrap();
        // dev.set_f32(chan, val)
        Ok(())
    }

    fn get_f32(&self, _chan: u16) -> Result<f32, String> {
        // let dev = self.output.read().unwrap();
        // TODO get from msg?
        // dev.get_f32(chan)
        unimplemented!()
    }

    fn sync(&mut self) -> Result<(), String> {
        unimplemented!()
        //let mut dev = self.output.write().unwrap();
        // dev.handle_msg(&self.current_msg)
        // dev.sync()
        // unimplemented!()
    }
}

// fn frame_to_msg(frame: Vec<f32>, msg: &mut Msg) {
//     for (i, v) in frame.iter().enumerate() {
//         msg.vals[i].1 = Val::F32(*v)
//     }
// }

impl MovingAverage {
    fn advance_frame(&mut self) {
        // let frame = msg_to_frame(&self.target_msg);
        let frame = Frame::from_msg(&self.target_msg);
        self.frames.push_back(frame);
        self.frames.pop_front();
    }

    fn num_chans(&self) -> usize {
        self.frames[0].num_chans()
    }

    // fn chan_avg(&self, cid: usize) -> f32 {
    //     let mut sum: f32 = 0.0;
    //     for frame in self.frames.iter() {
    //         sum += frame[cid]
    //     }
    //     sum / self.frames.len() as f32
    // }

    fn avg_frame(&self) -> Frame {
        let num_chans = self.num_chans();
        let mut result = Frame::new(num_chans); // Vec::with_capacity(self.frames[0].len());

        for frame in self.frames.iter() {
            for (i, v) in frame.vals.iter().enumerate() {
                result.vals[i] += v;
            }
        }

        let num_frames = self.frames.len();
        for item in result.vals.iter_mut().take(num_chans) {
            *item /= num_frames as f32;
        }
        result
    }

    fn current_frame(&self) -> Frame {
        Frame::from_msg(&self.current_msg)
    }

    fn has_reached_target(&self) -> bool {
        self.current_frame() == self.target_frame()
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

                // should I use target_msg's timestamp instead?
                // let now = Instant::now();
                // let progress = (now.saturating_duration_since(mov_avg.last_msg_target_time))
                //     .div_duration_f32(mov_avg.transition_period)
                //     .min(1.0).max(0.0);
                // println!("progress = {}", progress);

                // TODO: order is not guaranteed, have to actually find the value with correct ChanId
                //   I'm just being lazy here
                // let vals = mov_avg.target_msg.vals.clone();
                // let mut anything_changed = false;
                // for ChanVal(ChanId(cid), target_val) in vals.iter() {
                //     match target_val {
                //         Val::F32(target_val) => {
                //             let prev_val = mov_avg.current_msg.vals[*cid as usize].1
                //                 .get_f32().unwrap();

                //             // let new_val = (prev_val * (1.0 - progress) + target_val * progress);
                //             let new_val = mov_avg.chan_avg()
                //             if new_val != prev_val {
                //                 println!("{} -> {}", prev_val, new_val);
                //                 anything_changed = true;
                //             }

                //             let new_val =
                //                 ChanVal(ChanId(*cid),
                //                         Val::F32(new_val));

                //             mov_avg.current_msg.vals[*cid as usize] = new_val;
                //         }
                //         Val::U16(_) => {
                //             unimplemented!()
                //         }
                //     }
                // }

                // if anything_changed {

                let avg_frame = mov_avg.avg_frame();

                if !mov_avg.has_reached_target() {
                    avg_frame.to_msg(&mut mov_avg.current_msg);

                    let mut output = mov_avg.output.lock().unwrap();
                    match output.handle_msg(&mov_avg.current_msg) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("moving_average output.handle_msg err: {:?}", e);
                        }
                    }
                }
                // }
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
            // thread::sleep(frame_period);
        }
    }
}

impl MsgHandler for MovingAverage {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        println!("moving_average got msg");
        self.target_msg = msg.clone();
        self.last_msg_recv_time = Instant::now();
        self.last_msg_target_time = self.last_msg_recv_time + self.transition_period;
        Ok(())
    }

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
    #[allow(dead_code)]
    pub fn new(
        output: Arc<Mutex<dyn MsgHandler>>,
        frame_period: Duration,
        transition_period: Duration,
    ) -> Self {
        let num_chans: usize = {
            let dev = output.lock().unwrap();
            dev.chans().len()
            // dev.num_chans() as usize
        };

        let frames_num: usize = transition_period.div_duration_f32(frame_period).ceil() as usize;

        let mut frames: VecDeque<Frame> = VecDeque::with_capacity(frames_num);
        for _ in 0..frames_num {
            frames.push_back(Frame::new(num_chans));
        }

        //   I'm not sure if I actually need frames
        // for i in 0..frames.len() {
        //     frames.append(vec![0.0; num_chans]);
        // }

        let mut vals = Vec::with_capacity(num_chans);
        for i in 0..num_chans {
            vals.push(ChanVal(ChanId(i as u16), Val::F32(0.0)))
        }

        let msg = Msg {
            seq_num: 0,
            timestamp: SystemTime::now(),
            vals,
        };

        MovingAverage {
            frame_period,
            transition_period,
            frames,
            output,
            current_msg: msg.clone(),
            target_msg: msg,
            last_msg_recv_time: Instant::now(),
            last_msg_target_time: Instant::now(),
        }
    }

    fn target_frame(&self) -> Frame {
        Frame::from_msg(&self.target_msg)
    }
}
