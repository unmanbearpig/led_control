// use std::env;
use core::num::ParseFloatError;
use core::num::ParseIntError;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use crate::action::{ActionSpec};
use crate::chan::ChanConfig;
use crate::chan_spec::ChanSpec;
use crate::coord::Coord;
use crate::dev_stats;
use crate::srv;
use crate::init_devs;
use crate::template::Template;

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

const DEFAULT_CONFIG_PATH: &str = "/etc/led_ctl.yaml";

fn parse_ip_port(args: &[&str]) -> Result<(IpAddr, Option<u16>), String> {
    if args.is_empty() {
        return Err("no ip specified".to_string());
    }
    if args.len() > 2 {
        return Err(format!(
            "too many args for ip:port (1 or 2 are allowed): {}",
            args.join(":")
        ));
    }

    let ip: IpAddr = args[0]
        .parse()
        .map_err(|e| format!("parse_ip_port: IP parse error: {:?}", e))?;

    let port: Option<u16> = match args.len() {
        1 => None,
        2 => Some(
            args[1]
                .parse()
                .map_err(|e|
                         format!("parse_ip_port: port parse error: {:?}", e))?,
        ),
        _ => unreachable!(),
    };

    Ok((ip, port))
}

impl DevChanConfig {
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
    use crate::config::{DevChanConfig, DevConfig};

    #[test]
    fn parse_dev_arg() {
        assert!(DevChanConfig::parse("aoeustnh").is_err());
        assert_eq!(
            DevChanConfig::parse("usb"),
            Ok(DevChanConfig {
                dev: DevConfig::Usb,
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

fn print_help() {
    println!("
led_ctl (or rust_ctl, whatevers) [OPTIONS] ACTION
OPTIONS
  --help                         -- prints help

Configuration (by default use {default_config_path})
  --cfg filename.yaml         -- use config file `filename.yaml` instead of
                                 default config {default_config_path}
  --no-cfg                    -- Don't use {default_config_path} and instead use
                                 automatic configuration if possible

Specifying devices
  --dev udpv1:127.0.0.1       -- UDP version 1 protocol with default port
  --dev udpv2:127.0.0.1       -- UDP v2 with default port
  --dev udpv2:127.0.0.1:1234  -- UDP v2 with custom port
  --dev usb                   -- All usb devices

Actions:
  print parsed config:
    print_cfg
  list channels:
    ls

  Serve:
    srv              -- listen on 0.0.0.0 and default port
    srv 127.0.0.1    -- different ip, default port
    srv 0.0.0.0:1234 -- custom port

    web [ADDR[:PORT]] -- serve web UI at ADDR:PORT or at default addr and port

  Set values and exit:
    set f32 0.1                    -- set all chans to f32 value
    set f32 0.0,0.34,0.88888,0.333 -- set multiple values in chan id order
    set f32 1,r:.9                 -- set all to 1 except channels tagged 'r'
                                      which are set to 0.9
    set f32 1,2:.7                 -- set all to 1 except 2 which is set to 0.7

  Unimplemented (yet or ever. Or is it? I don't remember):
    set u16 123                    -- set raw u16 value to all chans
    set u16 123,0,334              -- set raw u16 value per channel

  Demos (demo DEMO_NAME [OPTIONAL_ARGUMENTS])
    test_seq -- Sequentially fade each LED in and out. Useful for testing
    glitch   -- Glitchy demo that only works in some configurations
                (don't remember which) but is kinda cool
    hello    -- AKA Disco in web UI. Sine all LEDs at random-ish frequencies at
                high brightness
    fade     -- Probably fade out everything, I don't remember. Not very useful.
    whoosh   -- Quick fade across all LEDs sequentially (I think, I also don't
                remember clearly what it does)

Other
    space    -- does something cool or not

", default_config_path=DEFAULT_CONFIG_PATH);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub action: Option<ActionSpec>,
    pub templates: Option<Vec<Template>>,
    pub devs: Vec<DevChanConfig>,
}

impl<'a> Config {
    fn from_file(filename: &str) -> Result<Self, String> {
        let mut file = File::open(filename).map_err(|e| format!("{:?}", e))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|e| format!("{:?}", e))?;

        let cfg: Config =
            serde_yaml::from_str(buf.as_ref())
            .map_err(|e| format!("parsing config: {:?}", e))?;
        Ok(cfg)
    }

    pub fn init_srv(&self) ->
            Result<Arc<Mutex<dev_stats::DevStats<srv::Srv>>>, String> {
        let devs = init_devs::init_devs(&self.devs[..])?; // dyn
        let mut srv = srv::Srv::new();
        for (dev, chancfg) in devs.into_iter() {
            srv.add_dev(dev, chancfg.map(|c| c.into_iter()));
        }

        let sync_srv = Arc::new(Mutex::new(srv));
        let dev_stats = dev_stats::DevStats::new(sync_srv);
        let sync_dev = Arc::new(Mutex::new(dev_stats));
        {
            let sync_dev = sync_dev.clone();
            dev_stats::start_mon(sync_dev, Duration::from_millis(500));
        }
        Ok(sync_dev)
    }

    pub fn from_args(mut args: env::Args) -> Result<Self, String> {
        let mut action: Option<ActionSpec> = None;
        let mut devs: Vec<DevChanConfig> = Vec::new();
        let mut cfg: Option<Config> = None;

        args.next(); // remove the executable name from args

        let mut skip_default_config: bool = false;

        loop {
            let arg = args.next();
            if arg.is_none() {
                print_help();
                break;
            }
            let arg = arg.unwrap();

            match arg.as_ref() {
                "--help" => {
                    print_help();
                },
                "--cfg" => {
                    let filename = args.next();
                    if filename.is_none() {
                        return Err("--cfg requires config filename"
                                   .to_string());
                    }
                    cfg = Some(Config::from_file(filename.unwrap().as_ref())?);
                }
                "--no-cfg" => {
                    skip_default_config = true;
                }
                "--dev" => {
                    let dev_arg = args.next();
                    if dev_arg.is_none() {
                        return Err("No device specified for --dev option"
                                   .to_string());
                    }
                    let dev_arg = dev_arg.unwrap();
                    devs.push(DevChanConfig::parse(dev_arg)?);
                }
                "ls" => action = Some(ActionSpec::ListChans),
                "print_cfg" => action = Some(ActionSpec::PrintConfig),
                "srv" => {
                    let listen_arg = args.next();
                    let (listen_ip, listen_port) = match listen_arg {
                        Some(arg) => {
                            let parts: Vec<&str> = arg.split(':').collect();
                            let (ip, port) =
                                parse_ip_port(&parts[0..2.min(parts.len())])?;
                            (Some(ip), port)
                        }
                        None => (None, None),
                    };

                    action = Some(ActionSpec::Srv {
                        listen_ip,
                        listen_port,
                    });

                    if args.len() != 0 {
                        return Err("too many args for srv".to_string());
                    }
                }
                "set" => {
                    let setarg = args.next();
                    if setarg.is_none() {
                        return Err(
                            "set requires an argument: either 'f32' or 'u16'"
                                .to_string());
                    }
                    let setarg = setarg.unwrap();
                    let chan_spec_arg = args.next();
                    if chan_spec_arg.is_none() {
                        return Err(format!("set {} requires chan spec argument",
                                           setarg));
                    }
                    let chan_spec_arg = chan_spec_arg.unwrap();

                    let chan_spec = match setarg.as_ref() {
                        "f32" => ChanSpec::parse_f32(chan_spec_arg.as_ref()),
                        "u16" => ChanSpec::parse_u16(chan_spec_arg.as_ref()),
                        other => {
                            return Err(format!(
                                "set only supports f32 and u16, got '{}'",
                                other))
                        }
                    }?;
                    action = Some(ActionSpec::Set(chan_spec));
                }
                "web" => {
                    action = Some(ActionSpec::Web {
                        listen_addr: args.next(),
                    })
                }
                "space" => {
                    let location = match args.next() {
                        None => {
                            return Err(
                                "space needs: location (x,y,z) \
                                 radius brightness".to_string()
                            )
                        }
                        Some(loc) => loc,
                    };

                    let loc_parts: Vec<&str> = location.split(',').collect();
                    if loc_parts.len() != 3 {
                        return Err("location coordinates needs to be x,y,z"
                                   .to_string());
                    }
                    let loc_parts: Vec<f32> = loc_parts
                        .iter()
                        .map(|x| x.parse().map_err(|e| format!("{:?}", e)))
                        .collect::<Result<Vec<f32>, String>>()?;

                    let radius = match args.next() {
                        None => return Err(
                            "space needs: location (x,y,z) radius (missing) \
                             brightness (missing)"
                                .to_string(),
                        ),
                        Some(br) => br,
                    };
                    let radius: f32 = radius
                        .parse()
                        .map_err(|e: ParseFloatError| return format!("{:?}", e))?;

                    let brightness = match args.next() {
                        None => {
                            return Err("space needs: location (x,y,z) \
                                        radius brightness (missing)"
                                .to_string())
                        }
                        Some(br) => br,
                    };
                    let brightness: f32 = brightness
                        .parse()
                        .map_err(|e: ParseFloatError|
                                 return format!("{:?}", e))?;

                    action = Some(ActionSpec::Space {
                        location: Coord {
                            x: loc_parts[0],
                            y: loc_parts[1],
                            z: loc_parts[2],
                        },
                        radius,
                        brightness,
                    })
                }
                "demo" => {
                    let demo_arg = args.next();
                    if demo_arg.is_none() {
                        return Err("demo requires an argument".to_string());
                    }
                    let demo_arg = demo_arg.unwrap();
                    match demo_arg.as_ref() {
                        "test_seq" => action = Some(ActionSpec::TestSeq),
                        "glitch" => action = Some(ActionSpec::Glitch),
                        "hello" => action = Some(ActionSpec::Hello),
                        "fade" => action = Some(ActionSpec::Fade),
                        "whoosh" => action = Some(ActionSpec::Whoosh),
                        other => return Err(
                            format!("demo \"{}\" does not exist", other)),
                    }
                }
                other => return Err(format!("Unknown arg \"{}\"", other)),
            }
        }

        if cfg.is_none() && !skip_default_config {
            match fs::metadata(DEFAULT_CONFIG_PATH) {
                Ok(_) => cfg = Some(Self::from_file(DEFAULT_CONFIG_PATH)?),
                Err(e) => {
                    if e.kind() != io::ErrorKind::NotFound {
                        return Err(
                            format!("Could not read {}: {:?}",
                                    DEFAULT_CONFIG_PATH, e));
                    }
                }
            }
        }

        let cfg = match cfg {
            Some(mut cfg) => {
                cfg.devs.extend(devs);
                Config {
                    action: action,
                    templates: cfg.templates,
                    devs: cfg.devs,
                }
            }
            None => {
                Config { action, devs, templates: None }
            }
        };

        Ok(cfg)
    }
}
