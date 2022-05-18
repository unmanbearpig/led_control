use core::num::ParseIntError;
use crate::chan::ChanConfig;
use crate::parse_ip_port::parse_ip_port;
use serde_derive::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub devs: Vec<DevChanConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevConfig {
    TestDev,
    Usb {
        /// Use only devices with this serial (iSerial)
        /// None means don't check serial (only 1 usb device allowed)
        serial: Option<String>,
        pwm_period: Option<u16>,
    },
    UdpV1(IpAddr, Option<u16>),
    UdpV2 {
        ip: IpAddr,
        port: u16,
        chans: u16, // assume we know number of chans upfront
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevChanConfig {
    pub dev: DevConfig,
    pub chans: Option<Vec<ChanConfig>>,
}

impl DevChanConfig {
    /// Parse cmd line argument passed to --devv
    pub fn parse<S: AsRef<str>>(string: S) -> Result<Self, String> {
        let parts: Vec<&str> = string.as_ref().split('@').collect();

        let chan_configs: Option<Vec<ChanConfig>> = match parts.len() {
            0 => unreachable!(),
            1 => None,
            2 => {
                let res: Result<Vec<u16>, ParseIntError> =
                    parts[1].split(',').map(|n| n.parse()).collect();
                let indexes = res.map_err(|e| e.to_string())?;
                Some(
                    indexes
                        .into_iter()
                        .map(|i| ChanConfig {
                            index: i,
                            ..Default::default()
                        })
                        .collect(),
                )
            }
            _ => return Err("too many @ in spec".to_string()),
        };

        let dev_parts: Vec<&str> = parts[0].split(':').collect();
        if dev_parts.is_empty() {
            return Err(format!("invalid device spec \"{}\"", string.as_ref()));
        }

        match dev_parts[0] {
            "testdev" => Ok(DevChanConfig {
                dev: DevConfig::TestDev,
                chans: chan_configs,
            }),
            "usb" => Ok(DevChanConfig {
                dev: DevConfig::Usb {
                    pwm_period: None,
                    serial: None,
                },
                chans: chan_configs,
            }),
            "udpv1" => {
                let (ip, maybe_port) =
                    parse_ip_port(&dev_parts[1..3.min(dev_parts.len())])?;
                Ok(DevChanConfig {
                    dev: DevConfig::UdpV1(ip, maybe_port),
                    chans: chan_configs,
                })
            }
            "udpv2" => {
                let (ip, maybe_port) =
                    parse_ip_port(&dev_parts[1..3.min(dev_parts.len())])?;
                let chans = 3; // TODO fix hardcoded
                Ok(DevChanConfig {
                    dev: DevConfig::UdpV2 {
                        ip,
                        port: maybe_port.unwrap_or(8932),
                        chans,
                    },
                    chans: chan_configs,
                })
            }
            other => Err(format!("invalid device type \"{}\"", other)),
        }
    }
}

#[cfg(test)]
mod dev_config_test {
    use crate::configuration::{DevChanConfig, DevConfig};

    #[test]
    fn parse_dev_arg() {
        assert!(DevChanConfig::parse("aoeustnh").is_err());
        assert_eq!(
            DevChanConfig::parse("usb"),
            Ok(DevChanConfig {
                dev: DevConfig::Usb { serial: None, pwm_period: None },
                chans: None
            })
        );
        assert_eq!(
            DevChanConfig::parse("udpv1:127.0.0.2"),
            Ok(DevChanConfig {
                dev: DevConfig::UdpV1("127.0.0.2".parse().unwrap(), None),
                chans: None
            })
        );
        assert_eq!(
            DevChanConfig::parse("udpv1:127.0.0.2:1234"),
            Ok(DevChanConfig {
                dev: DevConfig::UdpV1("127.0.0.2".parse().unwrap(), Some(1234)),
                chans: None
            })
        );
    }
}
