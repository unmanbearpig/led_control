use leds::srv::Srv;
use leds::action::Action;
use leds::chan_spec::ChanSpec;
use leds::udp_srv;
use leds::configuration::Configuration;

use std::net::IpAddr;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct ListenConf {
    pub listen_ip: Option<IpAddr>,
    pub listen_port: Option<u16>,
}
impl Action<'_> for ListenConf {
    fn perform(&self, config: &Configuration) -> Result<(), String> {
        let srv = Srv::init_from_config(&config)?;
        let mut udp = udp_srv::UdpSrv::new(
            self.listen_ip, self.listen_port, srv)?;
        udp.run();
        Ok(())
    }
}

