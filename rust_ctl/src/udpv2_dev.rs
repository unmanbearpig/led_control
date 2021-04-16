use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use crate::proto::{self, ChanId, ChanVal, Msg, Val};

use std::fmt;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::time;

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

impl DevNumChans for UdpV2Dev {
    fn num_chans(&self) -> u16 {
        self.msg.vals.len() as u16
    }
}

impl DevRead for UdpV2Dev {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        if chan as usize >= self.msg.vals.len() {
            return Err(format!(
                "chan {} out of bounds (0-{})",
                chan,
                self.msg.vals.len() - 1
            ));
        }

        match self.msg.vals[chan as usize].1 {
            Val::F32(v) => Ok(v),
            Val::U16(_) => Err(format!("no f32 value for chan {}", chan)),
        }
    }
}

impl DevWrite for UdpV2Dev {
    /// sets the internal state of the LED to the float value
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if chan >= self.num_chans() {
            return Err(format!(
                "UDPv2 set_f32: invalid chan {}, only 0-3 are allowed",
                chan
            ));
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
        self.msg.seq_num = self.msg.seq_num.wrapping_add(1);
        Ok(())
    }
}

impl Dev for UdpV2Dev {}

const DEFAULT_PORT: u16 = 8932;

impl UdpV2Dev {
    pub fn new(ip: IpAddr, port: Option<u16>, num_chans: u16) -> Result<Self, String> {
        let local_addr = "0.0.0.0:0";
        let port = port.unwrap_or(DEFAULT_PORT);
        let socket = UdpSocket::bind(local_addr).map_err(|e| format!("{}", e))?;
        socket.connect((ip, port)).expect("connect failed");

        Ok(UdpV2Dev {
            ip,
            port,
            socket,
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
