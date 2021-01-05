use std::net::IpAddr;
use std::time;

use serde_derive::{Serialize, Deserialize};
use crate::srv::{Srv};
use crate::proto;
use crate::config;
use crate::demo;
use crate::udp_srv;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    ListChans,
    PrintConfig,
    Srv {
        listen_ip: Option<IpAddr>,
        listen_port: Option<u16>,
    },

    SetSameU16(u16),
    SetAllU16(Vec<u16>),
    SetSameF32(f32),
    SetAllF32(Vec<f32>),

    DemoTestSeq,
    DemoGlitch,
    DemoHello,
    DemoFade,
    DemoWhoosh,
}

impl Action {
    pub fn perform(&self, srv: &mut Srv, config: &config::Config) -> Result<(), String> {
        match self {
            Action::PrintConfig => {
                println!("{}", serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e) )?);
                Ok(())
            }
            Action::ListChans => {
                println!("chans:");
                for (id, name) in srv.chans() {
                    println!("chan {} {}", id, name);
                }
                Ok(())
            }
            Action::SetSameF32(val) => {
                let mut msg = proto::Msg {
                    seq_num: 0,
                    timestamp: time::SystemTime::now(),
                    vals: Vec::new(),
                };

                for c in 0..srv.chans().len() {
                    msg.vals.push(
                        proto::ChanVal(proto::ChanId(c as u16),
                                       proto::Val::F32(*val)));
                }

                srv.handle_msg(&mut msg)
            }
            Action::SetAllF32(fvals) => {
                if fvals.len() != srv.chans().len() {
                    let msg = format!(
                        "we have {} chans but you've specified only {} values",
                        srv.chans().len(), fvals.len());
                    return Err(msg)
                }

                let vals = srv.chans().zip(fvals)
                    .map(|((cid, _), v)| proto::ChanVal(cid, proto::Val::F32(*v)))
                    .collect();

                let mut msg = proto::Msg {
                    seq_num: 0,
                    timestamp: time::SystemTime::now(),
                    vals: vals,
                };

                srv.handle_msg(&mut msg)
            }
            Action::SetSameU16(val) => {
                let mut msg = proto::Msg {
                    seq_num: 0,
                    timestamp: time::SystemTime::now(),
                    vals: Vec::new(),
                };

                for c in 0..srv.chans().len() {
                    msg.vals.push(
                        proto::ChanVal(proto::ChanId(c as u16),
                                       proto::Val::U16(*val)));
                }

                srv.handle_msg(&mut msg)
            }
            Action::DemoTestSeq => {
                demo::test_seq::run(srv)
            }
            Action::DemoGlitch => {
                demo::glitch::run(srv)
            }
            Action::DemoHello => {
                demo::hello::run(srv)
            }
            Action::DemoFade => {
                demo::fade::run(srv)
            }
            Action::DemoWhoosh => {
                demo::whoosh::run(srv)
            }
            Action::Srv { listen_ip: ip, listen_port: port } => {
                let mut udp = udp_srv::UdpSrv::new(*ip, *port)?;

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
    }
}
