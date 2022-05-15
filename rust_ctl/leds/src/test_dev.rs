use std::fmt;

use crate::frame::Frame;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};

#[derive(Debug)]
pub struct TestDev {
    frame: Frame<f32>,

    /// Frame counter for printing if enabled, `None` means disabled
    print_frame_num: Option<usize>,
}

impl TestDev {
    pub fn new(print_vals: bool) -> Self {
        let print_frame_num = if print_vals {
            Some(0)
        } else {
            None
        };

        TestDev { frame: Frame::new(3), print_frame_num }
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

        Ok(self.frame.get(chan).unwrap_or(0.0))
    }
}

impl DevWrite for TestDev {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        let res = self.frame.merge_frame(frame);

        if let Some(frame_num) = &mut self.print_frame_num {
            println!(" -- Frame {:7} -------------", *frame_num);
            self.frame.print_vals();
            println!();
            *frame_num += 1
        }
        res
    }
}

impl Dev for TestDev {}
