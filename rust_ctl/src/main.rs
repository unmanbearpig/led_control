#![feature(test, div_duration)]

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
mod task;
mod controller;
mod filters;
mod runner;
mod web_tiny;

#[macro_use]
extern crate rust_embed;

use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::chan::ChanConfig;

type DevConfList = Vec<(Box<dyn dev::Dev>, Option<Vec<ChanConfig>>)>;

fn init_devs(dev_configs: &[config::DevChanConfig]) ->
    Result<DevConfList, String> {
        let mut devs: DevConfList = Vec::new();

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
    let dev_stats = dev_stats::DevStats::new(sync_srv);
    let sync_dev = Arc::new(RwLock::new(dev_stats));
    {
        let sync_dev = sync_dev.clone();
        dev_stats::start_mon(sync_dev, Duration::from_millis(500));
    }

    // let filter = MovingAverage::new(
    //     sync_dev.clone(),
    //     Duration::from_millis(10),
    //     Duration::from_millis(3000));

    // let (_tx, _rx) = mpsc::channel::<TaskMsg>();

    // let filter =
    //     Arc::new(RwLock::new(filter));

    // let join_handle = {
    //     let filter = filter.clone();
    //     // let srv = sync_dev.clone();
    //     thread::spawn(move || {
    //         let res = Runner::run(filter, rx);
    //         res
    //     })
    // };

    // let task = Some(Task {
    //     name: "Hello task from web test".to_string(),
    //     chan: tx,
    //     join_handle: join_handle,
    // });

    config.action.perform(sync_dev, &config)?;

    Ok(())
}
