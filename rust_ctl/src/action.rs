use std::net::IpAddr;

use crate::actions;
use crate::chan_spec::ChanSpec;
use crate::config;
use crate::coord::Coord;
use crate::demo;
use crate::msg_handler::{MsgHandler};
use crate::udp_srv;
use crate::web_tiny;
use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod chan_spec_parse_test {
    use crate::chan_spec::{ChanSpec, ChanSpecGeneric};

    #[test]
    fn parse_all() {
        assert_eq!(
            ChanSpec::parse_f32(".4").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::SomeWithDefault(0.4, Vec::new()))
        )
    }

    #[test]
    fn parse_some_with_default() {
        assert_eq!(
            ChanSpec::parse_f32(".4,3:1.0").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::SomeWithDefault(
                0.4,
                vec![("3".to_string(), 1.0)]
            ))
        )
    }

    #[test]
    fn parse_some_1_arg() {
        assert_eq!(
            ChanSpec::parse_f32("1:.4").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Some(vec![("1".to_string(), 0.4)]))
        )
    }

    #[test]
    fn parse_some() {
        assert_eq!(
            ChanSpec::parse_f32("0:.4,2:.7").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Some(vec![
                ("0".to_string(), 0.4),
                ("2".to_string(), 0.7)
            ]))
        )
    }

    #[test]
    fn parse_each() {
        assert_eq!(
            ChanSpec::parse_f32(".4,.7").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Each(vec![0.4, 0.7]))
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
        listen_addr: Option<String>,
    },

    // location, radius, brightness
    Space(Coord, f32, f32),

    DemoTestSeq,
    DemoGlitch,
    DemoHello,
    DemoFade,
    DemoWhoosh,
    DemoFade2 {
        chan_spec: ChanSpec,
    },
}

impl Action {
    pub fn perform(
        &self,
        srv: Arc<Mutex<dyn MsgHandler>>,
        config: &config::Config,
    ) -> Result<(), String> {
        match self {
            Action::PrintConfig => {
                println!(
                    "{}",
                    serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e))?
                );
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

                demo::space::run(
                    srv,
                    demo::space::Config {
                        location: *loc,
                        radius: *radius,
                        brightness: *brightness,
                    },
                )
            }
            Action::Set(spec) => {
                actions::set::run(spec, srv)
            }
            Action::DemoTestSeq => demo::test_seq::run(srv),
            Action::DemoGlitch => demo::glitch::run(srv),
            Action::DemoHello => demo::hello::run(srv),
            Action::DemoFade => demo::fade::run(srv),
            Action::DemoWhoosh => demo::whoosh::run(srv),
            Action::Srv {
                listen_ip: ip,
                listen_port: port,
            } => {
                let mut udp = udp_srv::UdpSrv::new(*ip, *port, srv)?;
                udp.run();
                Ok(())
            }
            action => {
                eprintln!("action {:?} not implemented", action);
                unimplemented!();
            }
        }
    }
}
