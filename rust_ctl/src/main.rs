#![feature(test)]

mod proto;
mod usb;
mod srv;
mod dev;
mod old_proto;
mod cli;

// use std::io;
use std::net;
use std::process;

// inputs: new udp
// outputs: usb, old udp, new udp, spi (later?)

fn main() -> Result<(), String> {
    let mut srv = srv::Srv::new();
    let mut usb_devs = usb::UsbDev::find_devs()?;
    for dev in usb_devs.iter_mut() {
        srv.add_dev(dev);
    }

    for (chan_id, descr) in srv.chans().iter() {
        println!("chan: {} {}", chan_id, descr);
    }

    let listen_addr = "127.0.0.1:8732";
    let socket = net::UdpSocket::bind(listen_addr);
    if socket.is_err() {
        eprintln!("socket err: {}", socket.unwrap_err());
        process::exit(1);
    }
    let socket = socket.unwrap();

    let mut buf = [0u8; proto::MSG_MAX_SIZE];

    println!("listening on {}...", listen_addr);
    loop {
        let (len, addr) = socket.recv_from(&mut buf).unwrap();
        println!("read {} bytes from {}", len, addr);
        let msg = proto::Msg::deserialize(&buf[0..len]).unwrap();
        println!("parsed msg: {:?}", msg);
    }
}
