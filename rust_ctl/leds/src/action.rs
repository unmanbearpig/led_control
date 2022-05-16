use crate::configuration::Configuration;

pub trait Action<'a>: std::fmt::Debug {
    fn perform(&self, config: &Configuration) -> Result<(), String>;
}
