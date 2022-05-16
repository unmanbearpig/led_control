use serde_derive::{Deserialize, Serialize};
use leds::demo;
use leds::chan_spec::ChanSpec;
use leds::coord::Coord;
use leds::action::Action;
use crate::actions;

use std::time::Duration;
use std::net::IpAddr;

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
            ActionSpec::ListChans => Ok(Box::new(actions::ListChans)),
            ActionSpec::PrintConfig => Ok(Box::new(actions::PrintConfig)),
            ActionSpec::Srv { listen_ip, listen_port, } =>
                Ok(Box::new(actions::ListenConf { listen_ip: *listen_ip,
                    listen_port: *listen_port })),
            ActionSpec::Set(chan_spec) => Ok(Box::new(
                    actions::Set(chan_spec.clone()))),
            ActionSpec::Web { listen_addr } =>
                Ok(Box::new(actions::Web { listen_addr: listen_addr.clone() })),
            ActionSpec::Space { location, radius, brightness } =>
                Ok(Box::new(actions::Space {
                    location: *location, radius: *radius,
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
