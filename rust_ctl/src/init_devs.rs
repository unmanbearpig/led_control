use crate::dev;
use crate::config;
use std::sync::{Arc, Mutex};
use crate::chan::ChanConfig;
use crate::test_dev;
use crate::usb;
use crate::udpv1_dev;
use crate::udpv2_dev;


type DevConfList = Vec<(Arc<Mutex<dyn dev::Dev>>, Option<Vec<ChanConfig>>)>;

pub fn init_devs(dev_configs: &[config::DevChanConfig]) -> Result<DevConfList, String> {
    let mut devs: DevConfList = Vec::new();

    for devchanconfig in dev_configs.iter() {
        let devcfg = devchanconfig.dev;
        let chancfg: Option<Vec<ChanConfig>> = devchanconfig.chans.clone();

        match devcfg {
            config::DevConfig::TestDev => {
                devs.push((Arc::new(Mutex::new(test_dev::TestDev::new())), chancfg));
            }
            config::DevConfig::Usb { pwm_period }=> {
                for usbdev in usb::UsbDev::find_devs(pwm_period)? {
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
