#[allow(unused_variables)]

// use std::env;
use std::net::IpAddr;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use core::num::ParseIntError;
use core::num::ParseFloatError;

use serde_derive::{Serialize, Deserialize};

use crate::chan::ChanConfig;
use crate::action::{ChanSpec, Action};
use crate::coord::{Coord};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevConfig {
    TestDev,
    Usb,
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

const DEFAULT_CONFIG_PATH: &str = "/etc/led_ctl.yaml";

fn parse_ip_port(args: &[&str]) -> Result<(IpAddr, Option<u16>), String> {
    if args.len() == 0 {
        return Err("no ip specified".to_string())
    }
    if args.len() > 2 {
        return Err(
            format!("too many args for ip:port (1 or 2 are allowed): {}",
                    args.join(":")))
    }

    let ip: IpAddr = args[0].parse().map_err(|e| format!("parse_ip_port: {:?}", e))?;

    let port: Option<u16> = match args.len() {
        1 => None,
        2 => Some(args[1].parse().map_err(|e| format!("parse_ip_port: {:?}", e))?),
        _ => unreachable!()
    };

    Ok((ip, port))
}

impl DevChanConfig {
    pub fn parse<S: AsRef<str>>(string: S) -> Result<Self, String> {
        let parts: Vec<&str> = string.as_ref().split("@").collect();

        let chan_configs: Option<Vec<ChanConfig>> =
            match parts.len() {
                0 => unreachable!(),
                1 => {
                    None
                },
                2 => {
                    let res: Result<Vec<u16>, ParseIntError> =
                        parts[1]
                        .split(",")
                        .map(|n| n.parse())
                        .collect();
                    let indexes = res.map_err(|e| e.to_string())?;
                    Some(indexes.into_iter().map(|i| {
                        let mut cc = ChanConfig::default();
                        cc.index = i;
                        cc
                    }).collect())
                },
                _ => {
                    return Err("too many @ in spec".to_string())
                }
            };

        let dev_parts: Vec<&str> = parts[0].split(":").collect();
        if dev_parts.len() == 0 {
            return Err(format!("invalid device spec \"{}\"", string.as_ref()));
        }

        match dev_parts[0] {
            "testdev" => Ok(DevChanConfig { dev: DevConfig::TestDev, chans: chan_configs }),
            "usb" => Ok(DevChanConfig { dev: DevConfig::Usb, chans: chan_configs }),
            "udpv1" => {
                let (ip, maybe_port) = parse_ip_port(
                    &dev_parts[1..3.min(dev_parts.len())])?;
                Ok(DevChanConfig { dev: DevConfig::UdpV1(ip, maybe_port), chans: chan_configs })
            },
            "udpv2" => {
                let (ip, maybe_port) = parse_ip_port(
                    &dev_parts[1..3.min(dev_parts.len())])?;
                let chans = 3; // TODO fix hardcoded
                Ok(DevChanConfig {
                    dev: DevConfig::UdpV2 {
                        ip: ip,
                        port: maybe_port.unwrap_or(8932),
                        chans: chans
                    }, chans: chan_configs
                })
            }
            other => Err(format!("invalid device type \"{}\"", other))
        }
    }
}

#[cfg(test)]
mod dev_config_test {
    use crate::config::{DevChanConfig, DevConfig};

    #[test]
    fn parse_dev_arg() {
        assert!(DevChanConfig::parse("aoeustnh").is_err());
        assert_eq!(
            DevChanConfig::parse("usb"),
            Ok(DevChanConfig{ dev: DevConfig::Usb, chans: None })
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
/// cli
///   config (by default /etc/led_ctl.yaml is used):
///     --cfg config.yaml          -- use config.yaml
///     --no-cfg                   -- don't use default /etc/led_ctl.yaml
///
///   specifying devices:
///    --dev udpv1:127.0.0.1       -- udp version1 with default port
///    --dev udpv2:127.0.0.1       -- v2 with default port
///    --dev udpv2:127.0.0.1:1234  -- v2 with custom port
///    --dev usb                   -- all usb devices
///
///   actions:
///     list channels:
///       ls
///
///     serve:
///       srv              -- listen on 0.0.0.0 and default port
///       srv 127.0.0.1    -- different ip, default port
///       srv 0.0.0.0:1234 -- custom port
///
///     set values:
///       set f32 0.1                    -- set all chans to f32 value
///       set f32 0.0,0.34,0.88888,0.333 -- set multiple values in chan id order
///       set f32 1,r:.9                 -- set all to 1 except channels tagged 'r'
///                                         which are set to 0.9
///       set f32 1,2:.7                 -- set all to 1 except 2 which is set to 0.7
///     unimplemented:
///       set u16 123                    -- set raw u16 value to all chans
///       set u16 123,0,334              -- set raw u16 value per channel
///
///     demo
///       test_seq -- fade all LEDs in sequence
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub action: Action,
    pub devs: Vec<DevChanConfig>
}

impl Config {
    fn from_file(filename: &str) -> Result<Self, String> {
        let mut file = File::open(filename).map_err(|e| format!("{:?}", e))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|e| format!("{:?}", e))?;

        println!("config:\n{}", buf);

        let cfg: Config = serde_yaml::from_str(buf.as_ref())
            .map_err(|e| format!("parsing config: {:?}", e))?;
        Ok(cfg)
    }

    pub fn from_args(mut args: env::Args) -> Result<Self, String> {
        let mut action: Option<Action> = None;
        let mut devs: Vec<DevChanConfig> = Vec::new();
        let mut cfg: Option<Config> = None;

        args.next(); // remove the executable name from args

        let mut skip_default_config: bool = false;

        loop {
            let arg = args.next();
            if arg.is_none() {
                break;
            }
            let arg = arg.unwrap();

            match arg.as_ref() {
                "--cfg" => {
                    let filename = args.next();
                    if filename.is_none() {
                        return Err("--cfg requires config filename".to_string())
                    }
                    cfg = Some(Config::from_file(filename.unwrap().as_ref())?);
                }
                "--no-cfg" => {
                    skip_default_config = true;
                }
                "--dev" => {
                    let dev_arg = args.next();
                    if dev_arg.is_none() {
                        return Err("No device specified for --dev option".to_string())
                    }
                    let dev_arg = dev_arg.unwrap();
                    devs.push(DevChanConfig::parse(dev_arg)?);
                }
                "ls" => {
                    action = Some(Action::ListChans)
                }
                "print_cfg" => {
                    action = Some(Action::PrintConfig)
                }
                "srv" => {
                    let listen_arg = args.next();
                    let (listen_ip, listen_port) = match listen_arg {
                        Some(arg) => {
                            let parts: Vec<&str> = arg.split(":").collect();
                            let (ip, port) = parse_ip_port(&parts[1..3.min(parts.len())])?;
                            (Some(ip), port)
                        }
                        None => (None, None)
                    };

                    action = Some(Action::Srv {
                        listen_ip: listen_ip,
                        listen_port: listen_port,
                    });

                    if args.len() != 0 {
                        return Err("too many args for srv".to_string())
                    }
                }
                "set" => {
                    let setarg = args.next();
                    if setarg.is_none() {
                        return Err("set requires an argument: either 'f32' or 'u16'".to_string())
                    }
                    let setarg = setarg.unwrap();
                    let chan_spec_arg = args.next();
                    if chan_spec_arg.is_none() {
                        return Err(format!("set {} requires chan spec argument", setarg))
                    }
                    let chan_spec_arg = chan_spec_arg.unwrap();

                    let chan_spec = match setarg.as_ref() {
                        "f32" => {
                            ChanSpec::parse_f32(chan_spec_arg.as_ref())
                        }
                        "u16" => {
                            ChanSpec::parse_u16(chan_spec_arg.as_ref())
                        }
                        other => {
                            return Err(format!("set only supports f32 and u16, got '{}'", other))
                        }
                    }?;
                    action = Some(Action::Set(chan_spec));
                }
                "web" => {
                    action = Some(
                        Action::Web {
                            listen_addr: args.next(),
                        }
                    )
                }
                "space" => {
                    let location = match args.next() {
                        None => return Err(
                            "space needs: location (x,y,z) radius brightness".to_string()),
                        Some(loc) => loc
                    };

                    let loc_parts: Vec<&str> = location.split(",").collect();
                    if loc_parts.len() != 3 {
                        return Err("location coordinates needs to be x,y,z".to_string())
                    }
                    let loc_parts: Vec<f32> =
                        loc_parts.iter().map(|x| x.parse().map_err(|e| format!("{:?}", e)))
                        .collect::<Result<Vec<f32>, String>>()?;

                    let radius = match args.next() {
                        None => return Err(
                            "space needs: location (x,y,z) radius (missing) brightness (missing)"
                                .to_string()),
                        Some(br) => br
                    };
                    let radius: f32 = radius.parse()
                        .map_err(|e: ParseFloatError| return format!("{:?}", e))?;

                    let brightness = match args.next() {
                        None => return Err(
                            "space needs: location (x,y,z) radius brightness (missing)".to_string()),
                        Some(br) => br
                    };
                    let brightness: f32 = brightness.parse()
                        .map_err(|e: ParseFloatError| return format!("{:?}", e))?;

                    action = Some(
                        Action::Space(
                            Coord {
                                x: loc_parts[0],
                                y: loc_parts[1],
                                z: loc_parts[2],
                            },
                            radius,
                            brightness,
                        )
                    )

                }
                "demo" => {
                    let demo_arg = args.next();
                    if demo_arg.is_none() {
                        return Err("demo requires an argument".to_string())
                    }
                    let demo_arg = demo_arg.unwrap();
                    match demo_arg.as_ref() {
                        "test_seq" => action = Some(Action::DemoTestSeq),
                        "glitch" => action = Some(Action::DemoGlitch),
                        "hello" => action = Some(Action::DemoHello),
                        "fade" => action = Some(Action::DemoFade),
                        "whoosh" => action = Some(Action::DemoWhoosh),
                        other => return Err(format!(
                            "demo \"{}\" does not exist", other))
                    }
                }
                other => {
                    return Err(format!("Unknown arg \"{}\"", other))
                }
            }
        }

        if cfg.is_none() && !skip_default_config {
            match fs::metadata(DEFAULT_CONFIG_PATH) {
                Ok(_) => {
                    cfg = Some(Self::from_file(DEFAULT_CONFIG_PATH)?)
                },
                Err(e) => {
                    if e.kind() != io::ErrorKind::NotFound {
                        return Err(format!("Could not read {}: {:?}", DEFAULT_CONFIG_PATH, e))
                    }
                }
            }
        }


        let cfg = match cfg {
            Some(mut cfg) => {
                cfg.devs.extend(devs);
                Config {
                    action: action.unwrap_or(cfg.action),
                    devs: cfg.devs,
                }
            },
            None => {
                let action = match action {
                    Some(a) => a,
                    None => return Err("No action specified".to_string())
                };
                Config {
                    action: action,
                    devs: devs,
                }
            }
        };

        Ok(cfg)
    }
}
