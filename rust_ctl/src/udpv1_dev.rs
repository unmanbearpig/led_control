use crate::dev::Dev;
use crate::old_proto;


use std::fmt;
use std::net::IpAddr;
use std::net::UdpSocket;

pub struct UdpV1Dev {
    #[allow(dead_code)]
    ip: IpAddr,
    #[allow(dead_code)]
    port: u16,
    socket: UdpSocket,
    name: String,
    msg: old_proto::LedMsgF32,
}

impl fmt::Display for UdpV1Dev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Dev for UdpV1Dev {
    fn name(&self) -> &String {
        &self.name
    }

    fn num_chans(&self) -> u16 {
        4
    }

    /// sets the internal state of the LED to the float value
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if chan >= self.num_chans() {
            return Err(format!(
                "UDPv1 set_f32: invalid chan {}, only 0-3 are allowed",
                chan))
        }

        self.msg.values[chan as usize] = val;

        Ok(())
    }

    /// sends the set LED values to the device
    fn sync(&mut self) -> Result<(), String> {
        // eprintln!("UDPv1: sending msg {:?}...", self.msg);
        let bytes = &self.msg.into_slice();
        self.socket.send(bytes).expect("send failed");
        Ok(())
    }
}

const DEFAULT_PORT: u16 = 8932;

impl UdpV1Dev {
    pub fn new(ip: IpAddr, port: Option<u16>) -> Result<Self, String> {
        let local_addr = "0.0.0.0:0";
        let port = port.unwrap_or(DEFAULT_PORT);
        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| format!("{}", e))?;
        socket.connect((ip, port)).expect("connect failed");

        Ok(UdpV1Dev {
            ip: ip,
            port: port,
            socket: socket,
            name: format!("UDPv1 {}:{}", ip, port),
            msg: old_proto::LedMsgF32::default(),
        })
    }
}
