use core::num::ParseFloatError;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};

use serde_derive::{Deserialize, Serialize};

use crate::action_spec::ActionSpec;
use leds::chan_spec::ChanSpec;
use leds::coord::Coord;
use leds::template::Template;
use leds::configuration::{Configuration, DevChanConfig};
use leds::parse_ip_port::parse_ip_port;

const DEFAULT_CONFIG_PATH: &str = "/etc/led_ctl.yaml";

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
    pub templates: Option<Vec<Template>>,
    pub configuration: Configuration,
}

pub fn from_args(mut args: env::Args)
        -> Result<(Option<ActionSpec>, Config), String> {
    let mut action: Option<ActionSpec> = None;
    let mut configuration = Configuration::default();
    let mut cfg: Option<Config> = None;

    args.next(); // remove the executable name from args

    let mut skip_default_config: bool = false;

    loop {
        let arg = args.next();
        println!("arg = {arg:?}");

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
                configuration.devs.push(DevChanConfig::parse(dev_arg)?);
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
            Ok(_) => cfg = Some(Config::from_file(DEFAULT_CONFIG_PATH)?),
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
            cfg.configuration.devs.extend(configuration.devs);
            Config {
                templates: cfg.templates,
                configuration: cfg.configuration,
            }
        }
        None => {
            Config { configuration, templates: None }
        }
    };

    Ok((action, cfg))
}

impl Config {
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


}
