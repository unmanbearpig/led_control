use std::net::IpAddr;

use crate::actions;
use crate::chan_spec::ChanSpec;
use crate::chan_description::HasChanDescriptions;
use crate::config;
use crate::coord::Coord;
use crate::demo;
use crate::udp_srv;
use crate::web;
use crate::init_devs;
use crate::dev_stats;
use crate::srv;
use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        config: &config::Config,
    ) -> Result<(), String> {
        let init_srv = || -> Result<Arc<Mutex<dev_stats::DevStats<srv::Srv>>>, String> {
            let devs = init_devs::init_devs(&config.devs[..])?; // dyn
            let mut srv = srv::Srv::new();
            for (dev, chancfg) in devs.into_iter() {
                srv.add_dev(dev, chancfg.map(|c| c.into_iter()));
            }

            let sync_srv = Arc::new(Mutex::new(srv));
            let dev_stats = dev_stats::DevStats::new(sync_srv);
            let sync_dev = Arc::new(Mutex::new(dev_stats));
            {
                let sync_dev = sync_dev.clone();
                dev_stats::start_mon(sync_dev, Duration::from_millis(200));
            }
            Ok(sync_dev)
        };

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
                let srv = init_srv()?;
                let srv = srv.lock().map_err(|e| format!("{:?}", e))?;
                for descr in srv.chan_descriptions() {
                    let mut tags = String::new();
                    for tag in descr.tags.iter() {
                        tags += format!("{} ", tag.name()).as_ref();
                    }
                    println!("chan {} {} {}", descr.chan_id, descr.name, tags);
                }
                Ok(())
            }
            Action::Web { listen_addr } => {
                let listen_addr: Option<String> = listen_addr.clone();
                let config = config.clone();
                let mut web = web::Web::new(listen_addr)?;

                let srv = init_srv()?;
                web.run(srv, config)
            }
            Action::Space(loc, radius, brightness) => {
                println!("!!!!!!!!Hello from space!!!!!!!!!! (TODO)");

                let srv = init_srv()?;
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
                actions::set::run_msg(spec, init_srv()?)
            }
            Action::DemoTestSeq => demo::test_seq::run(init_srv()?),
            Action::DemoGlitch => demo::glitch::run(init_srv()?),
            Action::DemoHello => demo::hello::run(init_srv()?),
            Action::DemoFade => demo::fade::run(init_srv()?),
            Action::DemoWhoosh => demo::whoosh::run(init_srv()?),
            Action::Srv {
                listen_ip: ip,
                listen_port: port,
            } => {
                let srv = init_srv()?;
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
