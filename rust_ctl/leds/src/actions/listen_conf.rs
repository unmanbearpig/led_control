use crate::srv::Srv;
use crate::action::Action;
use crate::chan_spec::ChanSpec;
use crate::udp_srv;
use crate::configuration::Configuration;

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

