use crate::config;
use crate::producer::Producer;

pub trait Action<'a>: std::fmt::Debug {
    fn perform(&self, config: &config::Config) -> Result<(), String>;
}

impl<'a, T> Action<'_> for T
where T: Producer<'a> {
    fn perform(&self, config: &config::Config) -> Result<(), String> {
        todo!()
    }
}
