use std::net::IpAddr;

use serde_derive::{Serialize, Deserialize};
use crate::proto;
use crate::config;
use crate::demo;
use crate::udp_srv;
use crate::web_tiny;
use crate::msg_handler::{MsgHandler, ChanDescription};
use crate::coord::{Coord};
use crate::chan_spec::{ChanSpec};
// use crate::filters::moving_average::MovingAverage;
// use crate::task::{TaskMsg, Task};
// use crate::runner::Runner;
use std::sync::{Arc, Mutex};
// use std::sync::mpsc;
// use std::thread;
use std::time;

// use crate::srv::Srv;

#[cfg(test)]
mod chan_spec_parse_test {
    use crate::chan_spec::{ChanSpecGeneric, ChanSpec};

    #[test]
    fn parse_all() {
        assert_eq!(
            ChanSpec::parse_f32(".4").unwrap(),
            ChanSpec::F32(
                ChanSpecGeneric::SomeWithDefault(0.4, Vec::new())
            )
        )
    }

    #[test]
    fn parse_some_with_default() {
        assert_eq!(
            ChanSpec::parse_f32(".4,3:1.0").unwrap(),
            ChanSpec::F32(
                ChanSpecGeneric::SomeWithDefault(
                    0.4, vec![("3".to_string(), 1.0)])
            )
        )
    }


    #[test]
    fn parse_some_1_arg() {
        assert_eq!(
            ChanSpec::parse_f32("1:.4").unwrap(),
            ChanSpec::F32(
                ChanSpecGeneric::Some(vec![("1".to_string(), 0.4)])
            )
        )
    }


    #[test]
    fn parse_some() {
        assert_eq!(
            ChanSpec::parse_f32("0:.4,2:.7").unwrap(),
            ChanSpec::F32(
                ChanSpecGeneric::Some(
                    vec![("0".to_string(), 0.4), ("2".to_string(), 0.7)])
            )
        )
    }

    #[test]
    fn parse_each() {
        assert_eq!(
            ChanSpec::parse_f32(".4,.7").unwrap(),
            ChanSpec::F32(
                ChanSpecGeneric::Each(vec![0.4, 0.7])
            )
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    ListChans,
    PrintConfig,
    Srv {
        listen_ip: Option<IpAddr>,
        listen_port: Option<u16>,
    },

    Set(ChanSpec),

    Web {
        listen_addr: Option<String>
    },

    // location, radius, brightness
    Space(Coord, f32, f32),

    DemoTestSeq,
    DemoGlitch,
    DemoHello,
    DemoFade,
    DemoWhoosh,
    DemoFade2 {
        chan_spec: ChanSpec
    }
}

impl Action {
    pub fn perform(&self,
                   srv: Arc<Mutex<dyn MsgHandler>>,
                   config: &config::Config)
                   -> Result<(), String> {
        match self {
            Action::PrintConfig => {
                println!("{}", serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e) )?);
                Ok(())
            }
            Action::ListChans => {
                println!("chans:");
                let srv = srv.lock().map_err(|e| format!("{:?}", e))?;
                for (id, name) in srv.chans() {
                    println!("chan {} {}", id, name);
                }
                Ok(())
            }
            Action::Web { listen_addr } => {
                let listen_addr: Option<String> = listen_addr.clone();
                let config = config.clone();
                let mut web = web_tiny::Web::new(listen_addr)?;

                web.run(srv, config)
            }
            Action::Space(loc, radius, brightness) => {
                println!("!!!!!!!!Hello from space!!!!!!!!!! (TODO)");

                demo::space::run(srv, demo::space::Config {
                    location: *loc, radius: *radius, brightness: *brightness,
                })
            }
            Action::Set(spec) => {
                let mut srv = srv.lock().map_err(|e| format!("{:?}", e))?;

                match spec {
                    ChanSpec::F32(spec) => {
                        // need some ChanSpec(Generic?) method
                        // that will give us the values for each specified chan
                        let chan_descriptions: Vec<ChanDescription> =
                            srv.chan_descriptions();
                        let chanvals = spec.resolve_for_chans(chan_descriptions.as_slice())?;

                        let chanvals = chanvals.into_iter()
                            .map(|(cid, v)| proto::ChanVal(proto::ChanId(cid), proto::Val::F32(v)))
                            .collect();

                        let msg = proto::Msg {
                            seq_num: 0,
                            timestamp: time::SystemTime::now(),
                            vals: chanvals,
                        };

                        srv.handle_msg(&msg)
                    }
                    ChanSpec::U16(_) => unimplemented!()
                }
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
                            let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
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
