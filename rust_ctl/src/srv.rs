use crate::dev::{self, Dev};
use crate::chan::ChanConfig;
use crate::proto::{ChanId, ChanVal, Val, Msg};
use std::fmt::{self, Display, Formatter};

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
    cfg: ChanConfig,
}

struct SrvDev {
    dev: Box<dyn Dev>,
    dirty: bool,
}

pub struct Srv {
    devs: Vec<SrvDev>,
    chans: Vec<SrvChan>,
}

impl<'a> Srv {
    pub fn new() -> Self {
        Srv {
            devs: Vec::new(),
            chans: Vec::new(),
        }
    }

    pub fn add_dev<T>(&mut self, dev: Box<dyn dev::Dev>, chancfg: Option<T>) -> DevId
    where T: ExactSizeIterator<Item = ChanConfig>
    {
        let dev_id = DevId(self.devs.len() as u16);

        match chancfg {
            Some(chancfgs) => {
                if chancfgs.len() as u16 != dev.num_chans() {
                    panic!("invalid number of chans specified in chancfg: {} instead of {}",
                           chancfgs.len(), dev.num_chans());
                }

                for chan in chancfgs {
                    self.chans.push(SrvChan {
                        devid: dev_id,
                        cfg: chan,
                    })
                }
            }
            None => {
                for i in 0..dev.num_chans() {
                    let mut cc = ChanConfig::default();
                    cc.index = i;
                    self.chans.push(
                        SrvChan {
                            devid: dev_id,
                            cfg: cc,
                        }
                    );
                }
            }
        };

        self.devs.push(SrvDev { dev: dev, dirty: true });

        dev_id
    }

    pub fn chans(&'a self) -> impl ExactSizeIterator<Item = (ChanId, String)> + 'a {
        self.chans.as_slice()
            .into_iter()
            .enumerate()
            .map(move |(chan_id, SrvChan { devid, .. })| {
                let dev = self.get_dev(&devid);
                (ChanId(chan_id as u16),
                 format!("Chan {} {} \"{}\"", chan_id, devid, dev.name()))
            })
    }

    fn get_dev(&self, id: &DevId) -> &dyn dev::Dev {
        let DevId(idx) = id;
        self.devs[*idx as usize].dev.as_ref()
    }

    pub fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
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
}

impl Display for Srv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        for dev in self.devs.iter() {
            let dev = &dev.dev;
            res += format!("{} ", dev.name()).as_str();
        }
        write!(f, "Srv {}", res)
    }
}

fn adjust_chan_val(chan_cfg: &ChanConfig, val: f32) -> f32 {
    (chan_cfg.min +  (val as f64).powf(chan_cfg.exp) * (chan_cfg.max - chan_cfg.min)) as f32
}

impl Dev for Srv {
    fn name(&self) -> String {
        format!("Srv {}", self)
    }

    fn num_chans(&self) -> u16 {
        self.chans.len() as u16
    }

    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        let chan: &mut SrvChan = &mut self.chans[chan as usize];
        let val = adjust_chan_val(&chan.cfg, val);
        let dev = &mut self.devs[chan.devid.0 as usize];
        dev.dirty = true;
        dev.dev.set_f32(chan.cfg.index, val)?;

        Ok(())
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

        // sync all devs for now, optimize later
        for dev in devs {
            dev.as_mut().sync()?;
        }
        Ok(())
    }
}
