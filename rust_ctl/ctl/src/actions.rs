pub mod set;
pub mod list_chans;
pub use list_chans::ListChans;
pub mod listen_conf;
pub use listen_conf::ListenConf;
use crate::web;
use leds::srv::Srv;
use leds::chan_spec::ChanSpec;
use leds::action::Action;
use leds::coord::Coord;
use leds::demo;
use leds::configuration::Configuration;
use serde_derive::{Deserialize, Serialize};

// TODO the only thing that "needs" the full Config
#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct PrintConfig;
impl Action<'_> for PrintConfig {
    fn perform(&self, config: &Configuration) -> Result<(), String> {
        println!(
            "{}",
            serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e))?
        );
        Ok(())
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Web { pub listen_addr: Option<String> }
impl Action<'_> for Web {
    fn perform(&self, config: &Configuration) -> Result<(), String> {
        let config = config.clone();
        let mut web = web::Web::new(self.listen_addr.clone())?;

        let srv = Srv::init_from_config(&config)?;
        web.run(srv, config)
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Space {
    pub location: Coord,
    pub radius: f32,
    pub brightness: f32,
}
impl Action<'_> for Space {
    fn perform(&self, config: &Configuration) -> Result<(), String> {
        println!("!!!!!!!!Hello from space!!!!!!!!!! (TODO)");

        let srv = Srv::init_from_config(&config)?;
        demo::space::run(
            srv,
            demo::space::Config {
                location: self.location,
                radius: self.radius,
                brightness: self.brightness,
            },
        )
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Set(pub ChanSpec);
impl Action<'_> for Set {
    fn perform( &self, config: &Configuration) -> Result<(), String> {
        set::run_msg(
            &self.0,
            Srv::init_from_config(&config)?)
    }
}
