use crate::dev::Dev;
use crate::proto::{self, Msg, Val, ChanId, ChanVal};

use std::fmt;
use std::time;
use std::net::IpAddr;
use std::net::UdpSocket;

pub struct UdpV2Dev {
    ip: IpAddr,
    port: u16,
    socket: UdpSocket,
    msg: proto::Msg,
}

impl fmt::Display for UdpV2Dev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UDPv2 {}:{}", self.ip, self.port)
    }
}

impl Dev for UdpV2Dev {
    fn num_chans(&self) -> u16 {
        self.msg.vals.len() as u16
    }

    /// sets the internal state of the LED to the float value
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if chan >= self.num_chans() {
            return Err(format!(
                "UDPv2 set_f32: invalid chan {}, only 0-3 are allowed",
                chan))
        }

        self.msg.vals[chan as usize] = ChanVal(ChanId(chan), Val::F32(val));

        Ok(())
    }

    /// sends the set LED values to the device
    fn sync(&mut self) -> Result<(), String> {
        // eprintln!("UDPv2: sending msg {:?}...", self.msg);
        let mut bytes = [0u8; 1500];
        self.msg.serialize(&mut bytes);
        self.socket.send(&bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}

const DEFAULT_PORT: u16 = 8932;

impl UdpV2Dev {
    pub fn new(ip: IpAddr, port: Option<u16>, num_chans: u16) -> Result<Self, String> {
        let local_addr = "0.0.0.0:0";
        let port = port.unwrap_or(DEFAULT_PORT);
        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| format!("{}", e))?;
        socket.connect((ip, port)).expect("connect failed");

        Ok(UdpV2Dev {
            ip: ip,
            port: port,
            socket: socket,
            msg: Msg {
                seq_num: 0,
                timestamp: time::SystemTime::now(),
                vals: (0..num_chans)
                    .map(|cid| ChanVal(ChanId(cid), Val::F32(0.0)))
                    .collect(),
            },
        })
    }
}
