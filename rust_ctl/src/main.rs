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

use std::env;

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
    println!("config: {:?}", config);

    let devs = init_devs(&config.devs[..])?;
    println!("found {} devs", devs.len());

    let mut srv = srv::Srv::new();
    for (dev, chancfg) in devs.into_iter() {
        srv.add_dev(dev, chancfg.map(|c| c.into_iter()));
    }

    config.action.perform(&mut srv, &config)?;

    Ok(())
}
