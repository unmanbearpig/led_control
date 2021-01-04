use std::fmt::Display;

pub trait Dev
where
    Self: Display,
{
    fn name(&self) -> String;
    fn num_chans(&self) -> u16;
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String>;
    fn sync(&mut self) -> Result<(), String>;
}
