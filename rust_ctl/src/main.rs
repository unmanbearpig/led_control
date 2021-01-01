#![feature(test, slice_fill)]

mod proto;
mod usb;
mod srv;
mod dev;
mod old_proto;
mod config;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod demo;

use std::env;
use std::time;
use serde_yaml;

// inputs: new udp
// outputs: usb, old udp, new udp, spi (later?)

fn init_devs(dev_configs: &[config::DevChanConfig]) ->
    Result<Vec<(Box<dyn dev::Dev>, Option<Vec<u16>>)>, String> {

        let mut devs: Vec<(Box<dyn dev::Dev>, Option<Vec<u16>>)> = Vec::new();

        for config::DevChanConfig { dev: devcfg, chans: chancfg } in dev_configs.iter() {
            match devcfg {
                config::DevConfig::Usb => {
                    for usbdev in usb::UsbDev::find_devs()? {
                        devs.push(
                            (
                                Box::new(usbdev),
                                chancfg.clone(),
                            )
                        );
                    }
                }
                config::DevConfig::UdpV1(ip, port) => {
                    devs.push(
                        (
                            Box::new(udpv1_dev::UdpV1Dev::new(*ip, *port)?),
                            chancfg.clone(),
                        )
                    );
                }
                config::DevConfig::UdpV2 { ip, port, chans } => {
                    devs.push(
                        (
                            Box::new(udpv2_dev::UdpV2Dev::new(*ip, Some(*port), *chans)?),
                            chancfg.clone(),
                        )
                    );
                }
            }
        }

        Ok(devs)
    }

fn main() -> Result<(), String> {
    let config = config::Config::from_args(env::args())?;
    println!("config: {:?}", config);

    let devs = init_devs(&config.devs[..])?;
    println!("found {} devs", devs.len());
    // for d in devs.iter() {
    //     println!("{}", d.as_ref());
    // }

    let mut srv = srv::Srv::new();
    for (dev, chancfg) in devs.into_iter() {
        srv.add_dev(dev, chancfg);
    }

    match config.action {
        config::Action::PrintConfig => {
            println!("{}", serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e) )?);
        }
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
        config::Action::SetAllF32(fvals) => {
            if fvals.len() != srv.chans().len() {
                let msg = format!(
                    "we have {} chans but you've specified only {} values",
                    srv.chans().len(), fvals.len());
                return Err(msg)
            }

            let vals = srv.chans().iter().zip(fvals)
                .map(|((cid, _), v)| proto::ChanVal(*cid, proto::Val::F32(v)))
                .collect();

            let mut msg = proto::Msg {
                seq_num: 0,
                timestamp: time::SystemTime::now(),
                vals: vals,
            };
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
        config::Action::DemoWhoosh => {
            demo::whoosh::run(&mut srv)?;
        }
        config::Action::Srv { listen_ip: ip, listen_port: port } => {
            let mut udp = udp_srv::UdpSrv::new(ip, port)?;

            loop {
                match udp.recv() {
                    Ok(msg) => {
                        match srv.handle_msg(&msg) {
                            Ok(_) => continue,
                            Err(e) => eprintln!("Error handling msg: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("udp msg error: {}", e);
                    }
                }
            }
        }
        action => {
            eprintln!("action {:?} not implemented", action);
            unimplemented!();
        }
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
