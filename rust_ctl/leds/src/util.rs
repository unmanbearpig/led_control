use std::net::IpAddr;

pub fn parse_ip_port(args: &[&str]) -> Result<(IpAddr, Option<u16>), String> {
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
