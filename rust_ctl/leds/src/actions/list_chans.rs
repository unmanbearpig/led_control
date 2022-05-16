use crate::srv::Srv;
use crate::action::Action;
use crate::chan_spec::ChanSpec;
use crate::chan_description::HasChanDescriptions;
use serde_derive::{Deserialize, Serialize};
use crate::configuration::Configuration;

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct ListChans;
impl Action<'_> for ListChans {
    fn perform(&self, config: &Configuration) -> Result<(), String> {
        println!("chans:");
        let srv = Srv::init_from_config(&config)?;
        let srv = srv.lock().map_err(|e| format!("{:?}", e))?;
        for descr in srv.chan_descriptions() {
            let mut tags = String::new();
            for tag in descr.config.tags.iter() {
                tags += format!("{} ", tag.name()).as_ref();
            }
            println!("chan {} {} {}", descr.chan_id, descr.name, tags);
        }
        Ok(())
    }
}

