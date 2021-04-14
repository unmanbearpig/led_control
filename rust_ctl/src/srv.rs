use crate::dev::{self, Dev};
use crate::chan::ChanConfig;
use crate::proto::{ChanId, ChanVal, Val, Msg};
use crate::msg_handler::{MsgHandler, ChanDescription};
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DevId(u16);

impl Display for DevId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[Dev {}]", self.0)
    }
}

#[derive(Debug)]
struct SrvChan {
    devid: DevId,
    pub cfg: ChanConfig,
}

struct SrvDev {
    dev: Arc<Mutex<dyn Dev>>,
    dirty: bool,
}

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

impl<'a> Srv {
    pub fn new() -> Self {
        Srv {
            devs: Vec::new(),
            chans: Vec::new(),
        }
    }

    pub fn add_dev<T>(&mut self, dev: Arc<Mutex<dyn dev::Dev>>, chancfg: Option<T>) -> DevId
    where T: ExactSizeIterator<Item = ChanConfig>
    {
        let dev_id = DevId(self.devs.len() as u16);
        let num_chans = {
            let dev = dev.lock().unwrap();
            dev.num_chans()
        };

        match chancfg {
            Some(chancfgs) => {
                if chancfgs.len() as u16 != num_chans {
                    panic!("invalid number of chans specified in chancfg: {} instead of {}",
                           chancfgs.len(), num_chans);
                }

                for chan in chancfgs {
                    self.chans.push(SrvChan {
                        devid: dev_id,
                        cfg: chan,
                    })
                }
            }
            None => {

                for i in 0..num_chans {
                    let cc = ChanConfig {
                        index: i,
                        ..Default::default()
                    };

                    self.chans.push(
                        SrvChan {
                            devid: dev_id,
                            cfg: cc,
                        }
                    );
                }
            }
        };

        self.devs.push(SrvDev { dev, dirty: true });

        dev_id
    }

    fn get_dev(&self, id: &DevId) -> Arc<Mutex<dyn dev::Dev>> {
        let DevId(idx) = id;
        self.devs[*idx as usize].dev.clone()
    }
}

impl MsgHandler for Srv {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            match val {
                Val::F32(fval) => {
                    self.set_f32(*cid, *fval)?;
                },
                _ => unimplemented!(),
            }
        }

        self.sync()
    }

    fn chans(&self) -> Vec<(ChanId, String)> {
        self.chans.as_slice()
            .iter().enumerate()
            .map(|(chan_id, SrvChan { devid, .. })| {
                let dev = self.get_dev(&devid);
                let dev = dev.lock().unwrap();
                (ChanId(chan_id as u16),
                 format!("Chan {} {} \"{}\"", chan_id, devid, dev))
            }).collect()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        self.chans.iter().enumerate().map(|(cid, chan)| {
            ChanDescription {
                chan_id: cid as u16,
                name: format!("[cid: {}, dev {}, chan {}]",
                              cid, chan.devid.0, chan.cfg.index),
                tags: chan.cfg.tags.clone(),
                cuboid: chan.cfg.cuboid,
            }
        }).collect()
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

fn adjust_chan_val(chan_cfg: &ChanConfig, val: f32) -> f32 {
    (chan_cfg.min +  (val as f64).powf(chan_cfg.exp) * (chan_cfg.max - chan_cfg.min)) as f32
}

impl Dev for Srv {
    fn num_chans(&self) -> u16 {
        self.chans.len() as u16
    }

    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        let chan: &mut SrvChan = &mut self.chans[chan as usize];
        let val = adjust_chan_val(&chan.cfg, val);
        let dev = &mut self.devs[chan.devid.0 as usize];
        dev.dirty = true;
        let mut dev = dev.dev.lock().unwrap();
        dev.set_f32(chan.cfg.index, val)?;

        Ok(())
    }

    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        let chan: &SrvChan = &self.chans[chan as usize];
        let dev = &self.devs[chan.devid.0 as usize];

        let dev = dev.dev.lock().unwrap();
        dev.get_f32(chan.cfg.index)
    }

    fn sync(&mut self) -> Result<(), String> {
        // minimize the number of syncs to devices by skipping
        // the ones without dirty bit set,
        let devs = self.devs.iter_mut()
            .filter(|d| d.dirty)
            .map(|d| {
                d.dirty = false;
                &mut d.dev
            });

        for dev in devs {
            let mut dev = dev.lock().unwrap();
            dev.sync()?;
        }
        Ok(())
    }
}
