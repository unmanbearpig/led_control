use crate::coord::Coord;
use crate::msg_handler::MsgHandler;
use proto::v1::{ChanVal, Msg, Val};
use crate::wacom;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub location: Coord,
    pub radius: f32,
    pub brightness: f32,
}

pub fn run<T: MsgHandler + ?Sized>(srv: Arc<Mutex<T>>, conf: Config) -> Result<(), String> {
    let mut msg: Msg = {
        let srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;

        let chans = srv.chans();
        Msg {
            seq_num: 0,
            timestamp: time::SystemTime::now(),
            vals: chans
                .iter()
                .map(|(id, _)| (ChanVal(*id, Val::F32(0.0))))
                .collect(),
        }
    };

    let mut wacom_packet = wacom::WacomPacket::default();
    let mut wacom = wacom::Wacom::new("/dev/hidraw8")?;

    let delay = time::Duration::from_micros(2000);
    // let t = time::Instant::now();
    // let x_freq = 0.15;
    // let y_freq = 0.11;
    // let z_freq = 0.15;
    let mut loc = conf.location;
    loop {
        // let dt = t.elapsed().as_secs_f32();
        // loc.x = (dt * x_freq * std::f32::consts::PI * 2.0).sin();
        // loc.y = (dt * y_freq * std::f32::consts::PI * 2.0).sin();
        // loc.z = (dt * z_freq * std::f32::consts::PI * 2.0).cos();

        wacom.read(&mut wacom_packet)?;
        // y max: 34bc
        // x max: 5460
        // dist: 2a
        // pressure: ff

        loc.x = 1.0 - (wacom_packet.y as f32 / 0x34bc as f32).min(1.0);
        loc.y = (wacom_packet.x as f32 / 0x5460 as f32).min(1.0);
        loc.z = (wacom_packet.distance as f32 / 0x29 as f32).min(1.0);

        {
            let mut srv = srv.lock().map_err(|e| format!("write lock: {:?}", e))?;
            for (i, cuboid) in srv
                .chan_descriptions()
                .iter()
                .enumerate()
                .filter_map(|(i, cfg)| cfg.config.cuboid.map(|c| (i, c)))
            {
                // let intersection = cuboid.sphere_intersection(loc, conf.radius);
                // let result = (intersection * conf.brightness).min(1.0);

                let avg_dist = cuboid.avg_dist_to_point(loc);
                let result = ((conf.radius - avg_dist) * conf.brightness)
                    .min(1.0)
                    .max(0.0);

                // println!("led {} = {}; dist = {}", i, result, dist);
                msg.vals[i].1 = Val::F32(result);
            }

            msg.timestamp = time::SystemTime::now();
            msg.seq_num += 1;

            srv.handle_msg(&msg).expect("space: handle_msg error");
        }
        sleep(delay);
    }
}
