use crate::dev;
use crate::mux::{Config, DevConfig};
use std::sync::{Arc, Mutex};
use crate::chan::ChanConfig;
use crate::test_dev;
use crate::usb;
use crate::udpv1_dev;
use crate::udpv2_dev;


type DevConfList = Vec<(Arc<Mutex<dyn dev::Dev>>, Option<Vec<ChanConfig>>)>;

pub fn init_devs(configuration: &Config)
      -> Result<DevConfList, String> {
    let mut devs: DevConfList = Vec::new();

    for devchanconfig in configuration.devs.iter() {
        let devcfg = devchanconfig.dev.clone();
        let chancfg: Option<Vec<ChanConfig>> = devchanconfig.chans.clone();

        match devcfg {
            DevConfig::TestDev => {
                devs.push((Arc::new(Mutex::new(
                                test_dev::TestDev::new(true))), chancfg));
            }
            DevConfig::Usb { pwm_period, serial }=> {
                let dev = usb::UsbDev::find_dev(serial.as_deref(), pwm_period);
                let dev = match dev {
                    Ok(dev) => dev,
                    Err(e) => {
                        return Err(format!(
                                "Find USB device error (serial: {serial:?}): \
{:?}", e))
                    },

                };
                devs.push((Arc::new(Mutex::new(dev)), chancfg.clone()));
            }
            DevConfig::UdpV1(ip, port) => {
                devs.push((
                    Arc::new(Mutex::new(udpv1_dev::UdpV1Dev::new(ip, port)?)),
                    chancfg,
                ));
            }
            DevConfig::UdpV2 { ip, port, chans } => {
                devs.push((
                    Arc::new(Mutex::new(
                            udpv2_dev::UdpV2Dev::new(ip, Some(port), chans)?)),
                    chancfg,
                ));
            }
        }
    }

    Ok(devs)
}
