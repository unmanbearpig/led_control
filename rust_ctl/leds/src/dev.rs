use crate::frame::Frame;

use std::fmt::Display;

pub trait DevNumChans {
    fn num_chans(&self) -> u16;
}

pub trait DevRead
where
    Self: DevNumChans,
{
    fn get_f32(&self, chan: u16) -> Result<f32, String>;
    fn get_to_frame(&self, frame: &mut Frame<f32>) -> Result<(), String> {
        for chan in 0..self.num_chans() {
            frame.set(chan as u16, self.get_f32(chan)?);
        }
        Ok(())
    }
}

pub trait DevWrite
where
    Self: DevNumChans,
{
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String>;
}

pub trait Dev
where
    Self: DevNumChans + DevRead + DevWrite + Display + Send, {
}
