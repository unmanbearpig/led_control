
use std::net::{IpAddr, UdpSocket};

struct InitializedConfig {
    num_chans: u16,
}

pub struct UdpV3Dev {
    ip: IpAddr,
    port: u16,
    socket: UdpSocket,

    /// Received from the device
    initialized_config: Option<InitializedConfig>,
}
