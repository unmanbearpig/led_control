#![feature(test, slice_fill)]

mod proto;
mod usb;
mod srv;
mod dev;
mod old_proto;
mod config;
mod udp_srv;
mod udpv1_dev;
mod demo;

use std::env;
use std::time;

// inputs: new udp
// outputs: usb, old udp, new udp, spi (later?)

fn init_devs(dev_configs: &[config::DevConfig]) -> Result<Vec<Box<dyn dev::Dev>>, String> {
    let mut devs: Vec<Box<dyn dev::Dev>> = Vec::new();

    for devcfg in dev_configs.iter() {
        match devcfg {
            config::DevConfig::Usb => {
                for usbdev in usb::UsbDev::find_devs()? {
                    devs.push(Box::new(usbdev));
                }
            }
            config::DevConfig::UdpV1(ip, port) => {
                devs.push(Box::new(udpv1_dev::UdpV1Dev::new(*ip, *port)?));
            }
            _ => { unimplemented!() }
        }
    }

    Ok(devs)
}

fn main() -> Result<(), String> {
    let config = config::Config::from_args(env::args())?;
    println!("config: {:?}", config);

    let devs = init_devs(&config.devs[..])?;
    println!("found {} devs:", devs.len());
    for d in devs.iter() {
        println!("{}", d.as_ref());
    }

    let mut srv = srv::Srv::new();
    for dev in devs.into_iter() {
        srv.add_dev(dev);
    }

    match config.action {
        config::Action::ListChans => {
            println!("chans:");
            for (id, name) in srv.chans().iter() {
                println!("chan {} {}", id, name);
            }
        }
        config::Action::SetSameF32(val) => {
            let mut msg = proto::Msg {
                seq_num: 0,
                timestamp: time::SystemTime::now(),
                vals: Vec::new(),
            };

            for c in 0..srv.chans().len() {
                msg.vals.push(
                    proto::ChanVal(proto::ChanId(c as u16),
                                   proto::Val::F32(val)));
            }

            srv.handle_msg(&mut msg)?;
        }
        config::Action::SetSameU16(val) => {
            let mut msg = proto::Msg {
                seq_num: 0,
                timestamp: time::SystemTime::now(),
                vals: Vec::new(),
            };

            for c in 0..srv.chans().len() {
                msg.vals.push(
                    proto::ChanVal(proto::ChanId(c as u16),
                                   proto::Val::U16(val)));
            }

            srv.handle_msg(&mut msg)?;
        }

        config::Action::DemoTestSeq => {
            demo::test_seq::run(&mut srv)?;
        }
        config::Action::DemoGlitch => {
            demo::glitch::run(&mut srv)?;
        }
        config::Action::DemoHello => {
            demo::hello::run(&mut srv)?;
        }
        _ => unimplemented!()
    }

    Ok(())

    // let mut srv = srv::Srv::new();
    // let mut usb_devs = usb::UsbDev::find_devs()?;
    // for dev in usb_devs.iter_mut() {
    //     srv.add_dev(dev);
    // }

    // for (chan_id, descr) in srv.chans().iter() {
    //     println!("chan: {} {}", chan_id, descr);
    // }

    // let listen_addr = "127.0.0.1:8732";
    // let mut udp_srv = udp_srv::UdpSrv::new(listen_addr.to_string())?;

    // println!("listening on {}...", listen_addr);
    // loop {
    //     let msg = udp_srv.recv().unwrap();
    //     println!("parsed msg: {:?}", msg);
    // }
}
