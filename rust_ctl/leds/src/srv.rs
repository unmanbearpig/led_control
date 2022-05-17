use crate::frame::Frame;
use crate::chan::ChanConfig;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use crate::msg_handler::{MsgHandler};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use proto::v1::{ChanId, ChanVal, Msg, Val};
use crate::dev_stats;
use crate::init_devs;
use crate::configuration;
use std::time::Duration;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DevId(u16);

impl Display for DevId {
    fn fmt(&self, f: &mut Formatter<'_>)
            -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[Dev {}]", self.0)
    }
}

#[derive(Debug)]
struct SrvChan {
    devid: DevId,
    pub cfg: ChanConfig,
    prev_val_f32: f32,
}

struct SrvDev {
    dev: Arc<Mutex<dyn Dev>>,
    dirty: bool,
    frame: Frame<f32>,
}

#[derive(Default)]
pub struct Srv {
    devs: Vec<SrvDev>,
    chans: Vec<SrvChan>,
}

impl fmt::Debug for Srv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Srv")
            .field("devs", &format!("<{} devs>", self.devs.len()))
            .field("chans", &format!("<{} chans>", self.chans.len()))
            .finish()
    }
}

impl Srv {
    pub fn new() -> Self {
        Srv::default()
    }

    pub fn init_from_config(config: &configuration::Configuration) ->
            Result<Arc<Mutex<dev_stats::DevStats<Srv>>>, String> {
        let devs = init_devs::init_devs(config)?; // dyn
        let mut srv = Srv::new();
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

    pub fn add_dev<T>(
        &mut self, dev: Arc<Mutex<dyn Dev>>, chancfg: Option<T>
    ) -> DevId
    where
        T: ExactSizeIterator<Item = ChanConfig>,
    {
        let dev_id = DevId(self.devs.len() as u16);
        let num_chans = {
            let dev = dev.lock().unwrap();
            dev.num_chans()
        };

        match chancfg {
            Some(chancfgs) => {
                // Not checking because:
                // Config can override number of chans
                // because we don't know the number of channels for
                // udp protocol
                //
                // if chancfgs.len() as u16 != num_chans {
                //     panic!(
                //         "invalid number of chans specified in chancfg: {} instead of {}",
                //         chancfgs.len(),
                //         num_chans
                //     );
                // }

                for chan in chancfgs {
                    self.chans.push(SrvChan {
                        devid: dev_id,
                        cfg: chan,
                        prev_val_f32: 0.0,
                    })
                }
            }
            None => {
                for i in 0..num_chans {
                    let cc = ChanConfig {
                        index: i,
                        ..Default::default()
                    };

                    self.chans.push(SrvChan {
                        devid: dev_id,
                        cfg: cc,
                        prev_val_f32: 0.0,
                    });
                }
            }
        };

        let frame = Frame::empty();
        self.devs.push(SrvDev { dev, dirty: true, frame });

        dev_id
    }

    fn get_dev(&self, id: &DevId) -> Arc<Mutex<dyn Dev>> {
        let DevId(idx) = id;
        self.devs[*idx as usize].dev.clone()
    }
}

impl MsgHandler for Srv {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            match val {
                Val::F32(fval) => {
                    if *cid as usize >= self.chans.len() {
                        eprintln!("srv: chan {cid} out of bounds");
                        return Ok(())
                    }
                    let chan: &mut SrvChan = &mut self.chans[*cid as usize];
                    if *fval == chan.prev_val_f32 {
                        // skip it when trying to set to the previous value
                        continue;
                    }
                    chan.prev_val_f32 = *fval;
                    // TODO `val` not used? probably should be used
                    let val = chan.cfg.adjust_value(*fval);
                    let dev = &mut self.devs[chan.devid.0 as usize];
                    dev.dirty = true;
                    dev.frame.set(*cid, *fval);
                }
                _ => todo!(),
            }
        }

        for dev in self.devs.iter_mut() {
            {
                let mut locked = dev.dev.lock().unwrap();
                locked.set_frame(&dev.frame)?;
            }
            dev.dirty = false;
            dev.frame.clear();
        }

        Ok(())
    }
}

impl HasChanDescriptions for Srv {
    fn chans(&self) -> Vec<(ChanId, String)> {
        self.chans
            .as_slice()
            .iter()
            .enumerate()
            .map(|(chan_id, SrvChan { devid, .. })| {
                let dev = self.get_dev(devid);
                let dev = dev.lock().unwrap();
                (
                    ChanId(chan_id as u16),
                    format!("Chan {} {} \"{}\"", chan_id, devid, dev),
                )
            })
            .collect()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        self.chans
            .iter()
            .enumerate()
            .map(|(cid, chan)| {
                let name = format!(
                        "[cid: {}, dev {}, chan {}]",
                        cid, chan.devid.0, chan.cfg.index);

                ChanDescription::new(cid as u16, name, chan.cfg.clone())
            })
            .collect()
    }
}

impl Display for Srv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        for dev in self.devs.iter() {
            let dev = &dev.dev;
            let dev = dev.lock().unwrap();
            res += format!("{} ", dev).as_str();
        }
        write!(f, "Srv {}", res)
    }
}

impl DevNumChans for Srv {
    fn num_chans(&self) -> u16 {
        self.chans.len() as u16
    }
}

impl Srv {
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if chan as usize >= self.chans.len() {
            eprintln!("srv: chan {chan} out of bounds");
            return Ok(())
        }

        let chan: &mut SrvChan = &mut self.chans[chan as usize];
        // it doesn't work too well because of float precision
        // it things the value is changed when it didn't
        if val == chan.prev_val_f32 {
            // skip it when trying to set to the previous value
            return Ok(())
        }
        chan.prev_val_f32 = val;

        let val = chan.cfg.adjust_value(val);

        let dev = &mut self.devs[chan.devid.0 as usize];
        dev.dirty = true;
        dev.frame.set(chan.cfg.index, val);
        // let mut dev = dev.dev.lock().unwrap();
        // dev.set_f32(chan.cfg.index, val)?;

        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        // minimize the number of syncs to devices by skipping
        // the ones without dirty bit set,
        let devs = self.devs.iter_mut().filter(|d| d.dirty).map(|d| {
            d.dirty = false;
            d
        });

        for d in devs {
            let mut dev = d.dev.lock().unwrap();
            if let Err(e) = dev.set_frame(&d.frame) {
                eprintln!("srv set_frame: {e:?}");
                // should we report the error?
            }
        }
        Ok(())
    }
}

impl DevWrite for Srv {
    fn set_frame(&mut self, frame: &Frame<f32>) -> Result<(), String> {
        // eprintln!("Srv set_frame {frame:?}");
        for (cid, val) in frame.vals.iter().enumerate() {
            let cid = cid as u16;
            if let Some(val) = val {
                self.set_f32(cid, *val)?;
            }
        }
        self.sync()
    }
}

impl DevRead for Srv {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        let chan: &SrvChan = &self.chans[chan as usize];
        let dev = &self.devs[chan.devid.0 as usize];

        let dev = dev.dev.lock().unwrap();
        dev.get_f32(chan.cfg.index)
            .map(|val| chan.cfg.unadjust_value(val))
    }
}

impl Dev for Srv {}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;
    use crate::test_dev;
    use std::sync::{Arc, Mutex};
    use crate::chan::ChanConfig;

    #[bench]
    fn bench_srv_dev_with_chan_config(b: &mut Bencher) {
        let mut srv = Srv::new();
        let test_dev = test_dev::TestDev::new(false);
        let sync_dev = Arc::new(Mutex::new(test_dev));
        let chan_cfgs = vec![
            ChanConfig {
                index: 0, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },
            ChanConfig {
                index: 1, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },
            ChanConfig {
                index: 2, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },

        ];
        let chan_cfg = Some(chan_cfgs.into_iter());
        srv.add_dev(sync_dev, chan_cfg);

        b.iter(move || {
            srv.set_f32(0, 0.1).unwrap();
            srv.set_f32(1, 0.6).unwrap();
            srv.set_f32(2, 0.99).unwrap();
            srv.sync().unwrap();
        })
    }

    #[bench]
    fn bench_srv_dev_without_chan_config(b: &mut Bencher) {
        let mut srv = Srv::new();
        let test_dev = test_dev::TestDev::new(false);
        let sync_dev = Arc::new(Mutex::new(test_dev));

        let chan_cfg: Option<std::iter::Empty<ChanConfig>> = None;
        srv.add_dev(sync_dev, chan_cfg);

        b.iter(move || {
            srv.set_f32(0, 0.1).unwrap();
            srv.set_f32(1, 0.6).unwrap();
            srv.set_f32(2, 0.99).unwrap();
            srv.sync().unwrap();
        })
    }

        #[bench]
    fn bench_srv_handle_msg(b: &mut Bencher) {
        let mut srv = Srv::new();
        let test_dev = test_dev::TestDev::new(false);
        let sync_dev = Arc::new(Mutex::new(test_dev));
        let chan_cfgs = vec![
            ChanConfig {
                index: 0, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },
            ChanConfig {
                index: 1, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },
            ChanConfig {
                index: 2, min: 0.0, max: 1.0, exp: Some(2.2),
                tags: Vec::new(), cuboid: None,
                disco_config: None,
            },

        ]; // TODO: try with it too
        let chan_cfg = Some(chan_cfgs.into_iter());
        srv.add_dev(sync_dev, chan_cfg);

        let msg = Msg::new(0, vec![
            ChanVal(ChanId(0), Val::F32(0.1)),
            ChanVal(ChanId(1), Val::F32(0.6)),
            ChanVal(ChanId(2), Val::F32(0.99)),
        ]);

        b.iter(move || {
            srv.handle_msg(&msg)
        })
    }
}
