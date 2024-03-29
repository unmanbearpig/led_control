use crate::frame::Frame;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use proto::old_proto;

use std::fmt;
use std::net::IpAddr;
use std::net::UdpSocket;

pub struct UdpV1Dev {
    ip: IpAddr,
    port: u16,
    socket: UdpSocket,
    msg: old_proto::LedMsgF32,
}

impl fmt::Display for UdpV1Dev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UDPv1 {}:{}", self.ip, self.port)
    }
}

impl DevNumChans for UdpV1Dev {
    fn num_chans(&self) -> u16 {
        4
    }
}

impl DevRead for UdpV1Dev {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        if chan > 3 {
            return Err(format!("chan {} out of bounds (0-3)", chan));
        }

        Ok(self.msg.values[chan as usize])
    }
}

impl DevWrite for UdpV1Dev {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        if frame.vals.len() >= self.num_chans() as usize {
            return Err(format!(
                "UDPv1 set_f32: invalid chan {}, only 0-3 are allowed",
                frame.vals.len()
            ));
        }

        for (cid, val) in frame.vals.iter().enumerate() {
            let cid = cid as u16;
            if let Some(val) = val {
                self.msg.values[cid as usize] = *val;
            }
        }

        let bytes = &self.msg.as_slice();
        self.socket.send(bytes).expect("send failed");
        Ok(())
    }

    // /// sets the internal state of the LED to the float value
    // fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
    //     if chan >= self.num_chans() {
    //         return Err(format!(
    //             "UDPv1 set_f32: invalid chan {}, only 0-3 are allowed",
    //             chan
    //         ));
    //     }

    //     self.msg.values[chan as usize] = val;

    //     Ok(())
    // }

    // /// sends the set LED values to the device
    // fn sync(&mut self) -> Result<(), String> {
    //     // eprintln!("UDPv1: sending msg {:?}...", self.msg);
    //     let bytes = &self.msg.as_slice();
    //     self.socket.send(bytes).expect("send failed");
    //     Ok(())
    // }
}

impl Dev for UdpV1Dev {}

const DEFAULT_PORT: u16 = 8932;

impl UdpV1Dev {
    pub fn new(ip: IpAddr, port: Option<u16>) -> Result<Self, String> {
        let local_addr = "0.0.0.0:0";
        let port = port.unwrap_or(DEFAULT_PORT);
        let socket = UdpSocket::bind(local_addr).map_err(|e| format!("{}", e))?;
        socket.connect((ip, port)).expect("connect failed");

        Ok(UdpV1Dev {
            ip,
            port,
            socket,
            msg: old_proto::LedMsgF32::default(),
        })
    }
}
