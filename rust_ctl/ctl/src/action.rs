use std::net::IpAddr;
use std::time::Duration;

use leds::actions;
use leds::action_spec;
use leds::chan_spec::ChanSpec;
use leds::chan_description::HasChanDescriptions;
use leds::config;
use leds::coord::Coord;
use leds::demo;
use leds::udp_srv;
use leds::web;
use serde_derive::{Deserialize, Serialize};

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

pub trait Action<'a>: std::fmt::Debug {
    fn perform(&self, config: &config::Config) -> Result<(), String>;
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct PrintConfig;
impl Action<'_> for PrintConfig {
    fn perform(&self, config: &config::Config,) -> Result<(), String> {
        println!(
            "{}",
            serde_yaml::to_string(&config).map_err(|e| format!("{:?}", e))?
        );
        Ok(())
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Web { pub listen_addr: Option<String> }
impl Action<'_> for Web {
    fn perform(&self, config: &config::Config,) -> Result<(), String> {
        let config = config.clone();
        let mut web = web::Web::new(self.listen_addr.clone())?;

        let srv = config.init_srv()?;
        web.run(srv, config)
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Space {
    pub location: Coord,
    pub radius: f32,
    pub brightness: f32,
}
impl Action<'_> for Space {
    fn perform(&self, config: &config::Config) -> Result<(), String> {
        println!("!!!!!!!!Hello from space!!!!!!!!!! (TODO)");

        let srv = config.init_srv()?;
        demo::space::run(
            srv,
            demo::space::Config {
                location: self.location,
                radius: self.radius,
                brightness: self.brightness,
            },
        )
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct ListChans;
impl Action<'_> for ListChans {
    fn perform(&self, config: &config::Config) -> Result<(), String> {
        println!("chans:");
        let srv = config.init_srv()?;
        let srv = srv.lock().map_err(|e| format!("{:?}", e))?;
        for descr in srv.chan_descriptions() {
            let mut tags = String::new();
            for tag in descr.config.tags.iter() {
                tags += format!("{} ", tag.name()).as_ref();
            }
            println!("chan {} {} {}", descr.chan_id, descr.name, tags);
        }
        Ok(())
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Srv {
    listen_ip: Option<IpAddr>,
    listen_port: Option<u16>,
}
impl Action<'_> for Srv {
    fn perform(&self, config: &config::Config) -> Result<(), String> {
        let srv = config.init_srv()?;
        let mut udp = udp_srv::UdpSrv::new(
            self.listen_ip, self.listen_port, srv)?;
        udp.run();
        Ok(())
    }
}

#[derive(Clone, std::fmt::Debug, Serialize, Deserialize)]
pub struct Set(pub ChanSpec);
impl Action<'_> for Set {
    fn perform( &self, config: &config::Config) -> Result<(), String> {
        actions::set::run_msg(&self.0, config.init_srv()?)
    }
}
