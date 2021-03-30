use std::fmt::Display;

pub trait Dev
where
    Self: Display + Send + Sync
{
    fn num_chans(&self) -> u16;
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String>;
    fn get_f32(&self, chan: u16) -> Result<f32, String>;
    fn sync(&mut self) -> Result<(), String>;
}
