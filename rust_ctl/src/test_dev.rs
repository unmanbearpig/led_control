
use std::fmt;

use crate::dev::Dev;

#[derive(Debug)]
pub struct TestDev {
    vals: Vec<f32>,
}

impl TestDev {
    pub fn new() -> Self {
        TestDev {
            vals: vec![0.0; 3],
        }
    }
}

impl fmt::Display for TestDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "testdev ({} chans)", self.vals.len())
    }
}

impl Dev for TestDev {
    fn num_chans(&self) -> u16 {
        self.vals.len() as u16
    }

    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        self.vals[chan as usize] = val;
        Ok(())
    }

    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        if chan as usize >= self.vals.len() {
            return Err(format!(
                "chan {} out of bounds (0-{})",
                chan,
                self.vals.len() -1,
            ))
        }

        Ok(self.vals[chan as usize])
    }

    fn sync(&mut self) -> Result<(), String> {
        print!("test_dev sync:  ");
        for v in self.vals.iter() {
            print!("{:1.08}  ", v);
        }
        println!();
        Ok(())
    }
}
