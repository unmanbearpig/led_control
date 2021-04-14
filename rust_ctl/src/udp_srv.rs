use std::net;
use crate::proto;

#[allow(dead_code)]
pub struct UdpSrv {
    listen_ip: net::IpAddr,
    listen_port: u16,
    socket: net::UdpSocket,
    buf: [u8; proto::MSG_MAX_SIZE],
}

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: u16 =  8932;

impl UdpSrv {
    pub fn new(listen_ip: Option<net::IpAddr>, listen_port: Option<u16>) -> Result<Self, String> {
        let listen_ip = listen_ip.unwrap_or_else(|| DEFAULT_IP.parse().unwrap());
        let listen_port = listen_port.unwrap_or(DEFAULT_PORT);

        let socket = net::UdpSocket::bind((listen_ip, listen_port));
        if socket.is_err() {
            return Err(format!("UdpSrv new: {:?}", socket.unwrap_err()))
        }

        let socket = socket.unwrap();

        Ok(UdpSrv {
            listen_ip, listen_port, socket,
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
