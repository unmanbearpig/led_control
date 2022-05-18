use crate::frame::Frame;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use proto::v1::{ChanId, ChanVal, Msg, Val};

use std::fmt;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::time;

pub struct UdpV2Dev {
    ip: IpAddr,
    port: u16,
    socket: UdpSocket,
    msg: Msg,
    num_chans: u16,
}

impl fmt::Display for UdpV2Dev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UDPv2 {}:{}", self.ip, self.port)
    }
}

impl DevNumChans for UdpV2Dev {
    fn num_chans(&self) -> u16 {
        self.num_chans
    }
}

impl DevRead for UdpV2Dev {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        // We might have more than 3 chans
        // Ideally we should get rid of num_chans or make it detect the
        // actual number of chans
        //
        // if chan as usize >= self.msg.vals.len() {
        //     return Err(format!(
        //         "chan {} out of bounds (0-{})",
        //         chan,
        //         self.msg.vals.len() - 1
        //     ));
        // }

        match self.msg.vals[chan as usize].1 {
            Val::F32(v) => Ok(v),
            Val::U16(_) => Err(format!("no f32 value for chan {}", chan)),
        }
    }
}

impl DevWrite for UdpV2Dev {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        // eprintln!("UdpV2 set_frame {frame:?}");
        if frame.vals.len() > self.num_chans() as usize {
            return Err(format!(
                    "UDPv2 set_f32: invalid chan {}, only 0-3 are allowed",
                    frame.vals.len()));
        }
        for (cid, val) in frame.vals.iter().enumerate() {
            if let Some(val) = val {
                self.msg.vals[cid] =
                    ChanVal(ChanId(cid as u16), Val::F32(*val));
            }
        }

        // eprintln!("UDPv2: sending msg {:?}...", self.msg);
        let mut bytes = [0u8; 1500];
        // ignore previously set time, use the time just before
        // sending the message
        self.msg.timestamp = time::SystemTime::now();
        let size = self.msg.serialize(&mut bytes);
        self.socket.send(&bytes[0..size]).map_err(|e| e.to_string())?;
        self.msg.seq_num = self.msg.seq_num.wrapping_add(1);
        Ok(())
    }
}

impl Dev for UdpV2Dev {}

const DEFAULT_PORT: u16 = 8932;

impl UdpV2Dev {
    pub fn new(
        ip: IpAddr, port: Option<u16>, num_chans: u16
    ) -> Result<Self, String> {
        let local_addr = "0.0.0.0:0";
        let port = port.unwrap_or(DEFAULT_PORT);
        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| format!("{}", e))?;
        socket.connect((ip, port)).expect("connect failed");

        Ok(UdpV2Dev {
            ip,
            port,
            socket,
            num_chans,
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
