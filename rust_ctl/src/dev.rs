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
}

pub trait DevWrite
where
    Self: DevNumChans,
{
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String>;
    fn sync(&mut self) -> Result<(), String>;
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        let mut errors: Vec<String> = Vec::new();
        for (chan, val) in frame.iter_with_chans() {
            if let Err(err) = self.set_f32(chan, *val) {
                errors.push(format!("chan {} set to {} error: {}", chan, val, err));
            }
        }

        if errors.is_empty() {
            return Ok(());
        }

        let mut err_msg = String::new();
        for err in errors.iter() {
            err_msg += format!("{}\n", err).as_ref();
        }

        Err(err_msg)
    }
}

pub trait Dev
where
    Self: DevNumChans + DevRead + DevWrite + Display + Send,
{
}
