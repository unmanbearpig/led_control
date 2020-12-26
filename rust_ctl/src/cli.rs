
// use std::env;
use std::net::IpAddr;

#[derive(Debug)]
pub enum Action {
    Srv,
    SetSameU16(u16),
    SetAllU16(Vec<u16>),
    SetSameF32(f32),
    SetAllF32(Vec<f32>),
}

#[derive(Debug)]
pub enum DevConfig {
    Usb,
    UdpV1(IpAddr, u16),
    UdpV2 {
        ip: IpAddr,
        port: u16,
        chans: u16, // assume we know number of chans upfront
    },
}

#[derive(Debug)]
pub struct Config {
    // target: (IpAddr, u16),
    action: Action,
    devs: Vec<DevConfig>
}

// impl Action {
//     pub fn parse_from_args(name: &str, args: &mut env::Args) -> Result<Self, String> {
//         match name {
//             "setu16" => {
//                 let vals: Result<Vec<u16>, String> = args.next()
//                     .ok_or_else(|| format!("No value specified for action {}", name))?
//                     .split(",")
//                     .map(|v| u16::from_str_radix(v, 16).map_err(|e| format!("{:?}", e)))
//                     .collect();
//                 let vals = vals?;

//                 match vals.len() {
//                     1 => Ok(Action::SetAllU16([vals[0], vals[0], vals[0], vals[0]])),
//                     4 => Ok(Action::SetAllU16(vals.try_into().map_err(|e| format!("{:?}", e))?)),
//                     l => Err(format!(
//                         "invalid number of values for {}: {} instead of 1 or {}",
//                         name, l, NUM_VALUES))
//                 }
//             }
//             "setf32" => {
//                 let vals: Result<Vec<f32>, String> = args.next()
//                     .ok_or_else(|| format!("No value specified for action {}", name))?
//                     .split(",")
//                     .map(|v| v.parse().map_err(|e| format!("{:?}", e)))
//                     .collect();
//                 let vals = vals?;

//                 match vals.len() {
//                     1 => Ok(Action::SetAllF32([vals[0], vals[0], vals[0],vals[0]])),
//                     4 => Ok(Action::SetAllF32(vals.try_into().map_err(|e| format!("{:?}", e))?)),
//                     l => Err(format!(
//                         "invalid number of values for {}: {} instead of 1 or {}",
//                         name, l, NUM_VALUES))
//                 }
//             }
//             invalid => Err(format!("invalid action {}", invalid))
//         }
//     }
// }


// fn parse_args(mut args: &mut env::Args) -> Result<Config, String> {
//     // let mut config = Config::default();

//     let mut ip: IpAddr = "127.0.0.1".parse().unwrap();
//     let mut port: u16 = 8932;
//     let mut action: Option<Action> = None;

//     args.next();
//     loop {
//         let arg = args.next();
//         if arg.is_none() {
//             break;
//         }
//         let arg = arg.unwrap();
//         match arg.as_str() {
//             "--ip" => {
//                 let res: Result<IpAddr, String> = parse_arg_value("--ip", args);
//                 match res {
//                     Ok(newip) => ip = newip,
//                     Err(e) => return Err(e)
//                 }

//             }
//             "--port" => {
//                 let res: Result<u16, String> = parse_arg_value("--port", args);
//                 match res {
//                     Ok(newport) => port = newport,
//                     Err(e) => return Err(e),
//                 }
//             }
//             name => {
//                 action = Some(Action::parse_from_args(name, args)?);
//             }
//         }
//     }

//     if action.is_none() {
//         return Err("Action not specified".to_string())
//     }
//     let action = action.unwrap();

//     Ok(Config {
//         target: (ip, port),
//         action: action
//     })
// }

// fn main() -> io::Result<()> {
//     assert_eq!(32, std::mem::size_of::<LedMsg16>());
//     assert_eq!(32, std::mem::size_of::<LedMsgF32>());

//     let config = parse_args(&mut env::args());
//     let config = config.unwrap();
//     println!("config: {:?}", config);

//     let local_addr = "0.0.0.0:22345";
//     let mut socket = UdpSocket::bind(local_addr)?;
//     println!("connecting...");
//     socket.connect(config.target).expect("connect failed");
//     println!("connected");

//     match config.action {
//         Action::SetAllU16(vals) => {
//             let mut to_send = LedMsg16::default();
//             to_send.values = vals;
//             println!("sending msg {:?}...", to_send);
//             let bytes = &to_send.into_slice();
//             println!("{} bytes: ", bytes.len());
//             for b in bytes.iter() {
//                 print!("{:x} ", b);
//             }
//             print!("\n");
//             socket.send(bytes).expect("send failed");
//         }
//         Action::SetAllF32(vals) => {
//             let mut to_send = LedMsgF32::default();
//              to_send.values = vals;
//             println!("sending msg {:?}...", to_send);
//             let bytes = &to_send.into_slice();
//             println!("{} bytes: ", bytes.len());
//             for b in bytes.iter() {
//                 print!("{:x} ", b);
//             }
//             print!("\n");
//             socket.send(bytes).expect("send failed");
//         }
//     }


//     println!("done");

//     Ok(())
// }
