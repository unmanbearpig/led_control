
use std::fmt;

use crate::dev::Dev;

#[derive(Debug)]
pub struct TestDev {
    name: String,
    vals: Vec<f32>,
}

impl TestDev {
    pub fn new() -> Self {
        TestDev {
            name: "TestDev".to_string(),
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
    fn name(&self) -> String {
        self.name.clone()
    }

    fn num_chans(&self) -> u16 {
        self.vals.len() as u16
    }

    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        self.vals[chan as usize] = val;
        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        print!("test_dev sync:  ");
        for v in self.vals.iter() {
            print!("{:1.08}  ", v);
        }
        print!("\n");
        Ok(())
    }
}
