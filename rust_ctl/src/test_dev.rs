use std::fmt;

use crate::frame::Frame;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};

#[derive(Debug)]
pub struct TestDev {
    frame: Frame<f32>,

    /// Whether we should print the values on receiving new frame
    print_vals: bool,

    /// frame counter for printing (only used if print_vals is true)
    frame_num: usize,
}

impl TestDev {
    pub fn new(print_vals: bool) -> Self {
        TestDev { frame: Frame::new(3), print_vals: print_vals, frame_num: 0 }
    }
}

impl fmt::Display for TestDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "testdev ({} chans)", self.frame.num_chans())
    }
}

impl DevNumChans for TestDev {
    fn num_chans(&self) -> u16 {
        self.frame.num_chans() as u16
    }
}

impl DevRead for TestDev {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        if chan >= self.num_chans() {
            return Err(format!(
                "chan {} out of bounds (0-{})",
                chan,
                self.num_chans() - 1,
            ));
        }

        Ok(self.frame.get(chan).unwrap_or_else(|| { 0.0 }))
    }
}

impl DevWrite for TestDev {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        let res = self.frame.merge_frame(frame);

        if self.print_vals {
            println!(" -- Frame {} -------------", self.frame_num);
            self.frame.print_vals();
            println!("");
            self.frame_num += 1;
        }
        res
    }
}

impl Dev for TestDev {}
