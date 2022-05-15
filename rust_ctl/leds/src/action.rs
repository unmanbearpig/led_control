use std::net::IpAddr;
use std::time::Duration;

use crate::actions;
use crate::chan_spec::ChanSpec;
use crate::chan_description::HasChanDescriptions;
use crate::config;
use crate::coord::Coord;
use crate::demo;
use crate::udp_srv;
use crate::web;
use crate::srv::Srv;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSpec {
    ListChans,
    PrintConfig,
    Srv { listen_ip: Option<IpAddr>, listen_port: Option<u16> },
    Set(ChanSpec),
    Web { listen_addr: Option<String> },
    Space { location: Coord, radius: f32, brightness: f32 },
    TestSeq,
    Glitch,
    Hello,
    Fade,
    Whoosh,
}

impl ActionSpec {
    pub fn init(&self) -> Result<Box<dyn Action>, String> {
        match self {
            ActionSpec::ListChans => Ok(Box::new(ListChans)),
            ActionSpec::PrintConfig => Ok(Box::new(PrintConfig)),
            ActionSpec::Srv { listen_ip, listen_port, } =>
                Ok(Box::new(ListenConf { listen_ip: *listen_ip,
                    listen_port: *listen_port })),
            ActionSpec::Set(chan_spec) => Ok(Box::new(Set(chan_spec.clone()))),
            ActionSpec::Web { listen_addr } =>
                Ok(Box::new(Web { listen_addr: listen_addr.clone() })),
            ActionSpec::Space { location, radius, brightness } =>
                Ok(Box::new(Space { location: *location, radius: *radius,
                    brightness: *brightness })),
            ActionSpec::TestSeq => Ok(Box::new(demo::test_seq::TestSeq)),
            ActionSpec::Glitch => Ok(Box::new(demo::glitch::Glitch)),
            ActionSpec::Hello => Ok(Box::new(demo::hello::Hello)),
            ActionSpec::Fade => {
                Ok(Box::new(demo::fade::FadeSpec {
                    frame_duration: Duration::from_secs_f32(1.0 / 60.0),
                    fade_duration: Duration::from_secs_f32(1.0),
                }))
            },
            ActionSpec::Whoosh => Ok(Box::new(demo::whoosh::Whoosh)),
        }
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

        let srv = Srv::init_from_config(&config.configuration)?;
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

        let srv = Srv::init_from_config(&config.configuration)?;
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
        let srv = Srv::init_from_config(&config.configuration)?;
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
pub struct ListenConf {
    listen_ip: Option<IpAddr>,
    listen_port: Option<u16>,
}
impl Action<'_> for ListenConf {
    fn perform(&self, config: &config::Config) -> Result<(), String> {
        let srv = Srv::init_from_config(&config.configuration)?;
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
        actions::set::run_msg(
            &self.0,
            Srv::init_from_config(&config.configuration)?)
    }
}
