use std::net;
use crate::proto;

#[allow(dead_code)]
pub struct UdpSrv {
    listen_ip: net::IpAddr,
    listen_port: u16,
    socket: net::UdpSocket,
    buf: [u8; proto::MSG_MAX_SIZE],
}

impl UdpSrv {
    pub fn new(listen_ip: net::IpAddr, listen_port: u16) -> Result<Self, String> {
        let socket = net::UdpSocket::bind((listen_ip.clone(), listen_port));
        if socket.is_err() {
            return Err(format!("UdpSrv new: {:?}", socket.unwrap_err()))
        }

        let socket = socket.unwrap();

        Ok(UdpSrv {
            listen_ip: listen_ip.clone(),
            listen_port: listen_port,
            socket: socket,
            buf: [0; proto::MSG_MAX_SIZE],
        })
    }

    pub fn recv(&mut self) -> Result<proto::Msg, String> {
        let (len, _addr) = self.socket.recv_from(&mut self.buf).unwrap();
        // eprintln!("read {} bytes from {}", len, addr);
        let msg = proto::Msg::deserialize(&self.buf[0..len]).unwrap();
        // eprintln!("parsed msg: {:?}", msg);
        Ok(msg)
    }
}
