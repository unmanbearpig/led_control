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
}

pub trait Dev
where
    Self: DevNumChans + DevRead + DevWrite + Display + Send,
{
}
