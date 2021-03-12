#![feature(test, slice_fill)]


mod proto;
mod usb;
mod srv;
mod dev;
mod chan;
mod old_proto;
mod config;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod test_dev;
mod demo;
mod action;
mod dev_stats;
mod msg_handler;
mod coord;
mod cuboid;
mod chan_spec;
mod wacom;
mod web;
mod task;

#[macro_use]
extern crate mime;

use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::chan::ChanConfig;

fn init_devs(dev_configs: &[config::DevChanConfig]) ->
    Result<Vec<(Box<dyn dev::Dev>, Option<Vec<ChanConfig>>)>, String> {

        let mut devs: Vec<(Box<dyn dev::Dev>, Option<Vec<ChanConfig>>)> = Vec::new();

        for devchanconfig in dev_configs.iter() {
            let devcfg = devchanconfig.dev;
            let chancfg: Option<Vec<ChanConfig>> = devchanconfig.chans.clone();

            match devcfg {
                config::DevConfig::TestDev => {
                    devs.push(
                        (
                            Box::new(test_dev::TestDev::new()),
                            chancfg,
                        )
                    );
                }
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
                            Box::new(udpv1_dev::UdpV1Dev::new(ip, port)?),
                            chancfg,
                        )
                    );
                }
                config::DevConfig::UdpV2 { ip, port, chans } => {
                    devs.push(
                        (
                            Box::new(udpv2_dev::UdpV2Dev::new(ip, Some(port), chans)?),
                            chancfg,
                        )
                    );
                }
            }
        }

        Ok(devs)
    }

fn main() -> Result<(), String> {
    let config = config::Config::from_args(env::args())?;
    let devs = init_devs(&config.devs[..])?;
    let mut srv = srv::Srv::new();
    for (dev, chancfg) in devs.into_iter() {
        srv.add_dev(dev, chancfg.map(|c| c.into_iter()));
    }

    let sync_srv = Arc::new(RwLock::new(srv));
    let dev_stats = dev_stats::DevStats::new(sync_srv.clone());
    let sync_dev = Arc::new(RwLock::new(dev_stats));
    {
        let sync_dev = sync_dev.clone();
        dev_stats::start_mon(sync_dev, Duration::from_millis(500));
    }
    config.action.perform(sync_dev.clone(), &config)?;

    Ok(())
}
