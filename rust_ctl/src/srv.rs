use crate::tag::Tag;
use crate::chan::ChanConfig;
use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use crate::msg_handler::{MsgHandler};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use crate::proto::{ChanId, ChanVal, Msg, Val};
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

    pub fn add_dev<T>(&mut self, dev: Arc<Mutex<dyn Dev>>, chancfg: Option<T>) -> DevId
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
                if chancfgs.len() as u16 != num_chans {
                    panic!(
                        "invalid number of chans specified in chancfg: {} instead of {}",
                        chancfgs.len(),
                        num_chans
                    );
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

                    self.chans.push(SrvChan {
                        devid: dev_id,
                        cfg: cc,
                    });
                }
            }
        };

        self.devs.push(SrvDev { dev, dirty: true });

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
                    self.set_f32(*cid, *fval)?;
                }
                _ => unimplemented!(),
            }
        }

        self.sync()
    }
}

impl HasChanDescriptions for Srv {
    fn chans(&self) -> Vec<(ChanId, String)> {
        self.chans
            .as_slice()
            .iter()
            .enumerate()
            .map(|(chan_id, SrvChan { devid, .. })| {
                let dev = self.get_dev(&devid);
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
                ChanDescription::new(
                    cid as u16,
                    format!(
                        "[cid: {}, dev {}, chan {}]",
                        cid, chan.devid.0, chan.cfg.index
                    ),
                    chan.cfg.tags.iter().map(|t| Tag::new(t)).collect(), // actually need clone?
                    chan.cfg.cuboid,
                )
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

impl DevWrite for Srv {
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        let chan: &mut SrvChan = &mut self.chans[chan as usize];
        let val = chan.cfg.adjust_value(val);
        let dev = &mut self.devs[chan.devid.0 as usize];
        dev.dirty = true;
        let mut dev = dev.dev.lock().unwrap();
        dev.set_f32(chan.cfg.index, val)?;

        Ok(())
    }

    fn sync(&mut self) -> Result<(), String> {
        // minimize the number of syncs to devices by skipping
        // the ones without dirty bit set,
        let devs = self.devs.iter_mut().filter(|d| d.dirty).map(|d| {
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
