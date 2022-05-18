use serde_derive::{Deserialize, Serialize};
use leds::demo;
use leds::chan_spec::ChanSpec;
use leds::coord::Coord;
use leds::mux_config::MuxConfig;
use crate::actions;

use std::time::Duration;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSpec {
    ListChans,
    PrintConfig,
    Srv { listen_ip: Option<IpAddr>, listen_port: Option<u16> },
    SrvV3 { listen_ip: Option<IpAddr>, listen_port: Option<u16> },
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
    pub fn run(&self, config: &MuxConfig) -> Result<(), String> {
        let mux = leds::mux::Mux::init_from_config(config)?;

        match self {
            ActionSpec::ListChans => {
                use leds::chan_description::HasChanDescriptions;
                let mux = mux.lock().unwrap();
                for descr in mux.chan_descriptions() {
                    let mut tags = String::new();
                    for tag in descr.config.tags.iter() {
                        tags += format!("{} ", tag.name()).as_ref();
                    }
                    println!("chan {} {} {}", descr.chan_id, descr.name, tags);
                }
                Ok(())
            },
            ActionSpec::PrintConfig => {
                println!(
                    "{}",
                    serde_yaml::to_string(&config)
                        .map_err(|e| format!("{:?}", e))?);
                Ok(())
            },
            ActionSpec::Srv { listen_ip, listen_port } => {
                use leds::udp_srv::UdpSrv;
                let mut udp = UdpSrv::new(*listen_ip, *listen_port, mux)?;
                udp.run();
                Ok(())
            },
            ActionSpec::SrvV3 { listen_ip, listen_port } => {
                use leds::udp_srv_v3::UdpSrvV3;
                let mut udp = UdpSrvV3::new(*listen_ip, *listen_port, mux)?;
                udp.run();
                Ok(())
            },
            ActionSpec::Web { listen_addr } => {
                let mut web = crate::web::Web::new(listen_addr.clone())?;
                web.run(mux, config.clone())
            },
            ActionSpec::Set(cs) => actions::set::run_msg(cs, mux),
            ActionSpec::TestSeq => demo::test_seq::run(mux),
            ActionSpec::Glitch => demo::glitch::run(mux),
            ActionSpec::Whoosh => demo::whoosh::run(mux),
            ActionSpec::Hello => demo::hello::run(mux),
            ActionSpec::Fade => {
                use leds::runner::Runner;
                use leds::task::TaskMsg;
                use std::sync::mpsc;
                use leds::demo::fade::{Fade, FadeSpec};

                let (_sender, receiver) = mpsc::channel::<TaskMsg>();

                let fade_cfg = FadeSpec {
                    frame_duration: Duration::from_secs_f32(1.0 / 60.0),
                    fade_duration: Duration::from_secs_f32(1.0),
                };
                let fade = Fade::new(mux, fade_cfg);
                Fade::run(Arc::new(Mutex::new(fade)), receiver)
            },
            ActionSpec::Space { location, radius, brightness } => {
                demo::space::run(
                    mux,
                    demo::space::Config {
                        location:   *location,
                        radius:     *radius,
                        brightness: *brightness })
            },
        }
    }
}
