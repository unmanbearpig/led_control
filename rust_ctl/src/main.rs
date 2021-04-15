#![feature(test, div_duration)]

mod action;
mod chan;
mod chan_spec;
mod config;
mod controller;
mod coord;
mod cuboid;
mod demo;
mod dev;
mod dev_stats;
mod filters;
mod msg_handler;
mod old_proto;
mod proto;
mod runner;
mod srv;
mod task;
mod term_bar;
mod test_dev;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod usb;
mod wacom;
mod web_tiny;

#[macro_use]
extern crate rust_embed;

use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::chan::ChanConfig;

type DevConfList = Vec<(Arc<Mutex<dyn dev::Dev>>, Option<Vec<ChanConfig>>)>;

fn init_devs(dev_configs: &[config::DevChanConfig]) -> Result<DevConfList, String> {
    let mut devs: DevConfList = Vec::new();

    for devchanconfig in dev_configs.iter() {
        let devcfg = devchanconfig.dev;
        let chancfg: Option<Vec<ChanConfig>> = devchanconfig.chans.clone();

        match devcfg {
            config::DevConfig::TestDev => {
                devs.push((Arc::new(Mutex::new(test_dev::TestDev::new())), chancfg));
            }
            config::DevConfig::Usb => {
                for usbdev in usb::UsbDev::find_devs()? {
                    devs.push((Arc::new(Mutex::new(usbdev)), chancfg.clone()));
                }
            }
            config::DevConfig::UdpV1(ip, port) => {
                devs.push((
                    Arc::new(Mutex::new(udpv1_dev::UdpV1Dev::new(ip, port)?)),
                    chancfg,
                ));
            }
            config::DevConfig::UdpV2 { ip, port, chans } => {
                devs.push((
                    Arc::new(Mutex::new(udpv2_dev::UdpV2Dev::new(ip, Some(port), chans)?)),
                    chancfg,
                ));
            }
        }
    }

    Ok(devs)
}

fn main() -> Result<(), String> {
    // // working originally
    let config = config::Config::from_args(env::args())?;
    let devs = init_devs(&config.devs[..])?; // dyn
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

    config.action.perform(sync_dev, &config)?;

    Ok(())
}
