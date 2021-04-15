use std::sync::{Arc, Mutex};
use std::net;
use crate::proto;
use crate::msg_handler::MsgHandler;

#[allow(dead_code)]
pub struct UdpSrv {
    listen_ip: net::IpAddr,
    listen_port: u16,
    socket: net::UdpSocket,
    buf: [u8; proto::MSG_MAX_SIZE],
    output: Arc<Mutex<dyn MsgHandler>>,
}

const DEFAULT_IP: &str = "0.0.0.0";
const DEFAULT_PORT: u16 =  8932;

impl UdpSrv {
    pub fn new(listen_ip: Option<net::IpAddr>,
               listen_port: Option<u16>,
               output: Arc<Mutex<dyn MsgHandler>>)
               -> Result<Self, String> {
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
            output,
        })
    }

    pub fn recv(&mut self) -> Result<proto::Msg, String> {
        let (len, _addr) = self.socket.recv_from(&mut self.buf).unwrap();
        // eprintln!("read {} bytes from {}", len, addr);
        let msg = proto::Msg::deserialize(&self.buf[0..len]).unwrap();
        // eprintln!("parsed msg: {:?}", msg);
        Ok(msg)
    }

    pub fn run(&mut self) {
        loop {
            match self.recv() {
                Ok(msg) => {
                    let output = self.output.clone();
                    // let mut srv = output.lock().map_err(|e| format!("write lock: {:?}", e))?;
                    let mut output = match output.lock() {
                        Ok(output) => output,
                        Err(err) => {
                            eprintln!("UdpSrv mutex lock error: {}", err);
                            continue;
                        }
                    };

                    match output.handle_msg(&msg) {
                        Ok(_) => continue,
                        Err(e) => eprintln!("Error handling msg: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("UdpSrv recv error: {}", e);
                }
            }
        }
    }
}
