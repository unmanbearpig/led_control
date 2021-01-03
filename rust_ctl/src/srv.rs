use crate::dev;
use crate::chan::ChanConfig;
use crate::proto::{ChanId, ChanVal, Val, Msg};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevId(u16);

impl Display for DevId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[Dev {}]", self.0)
    }
}

pub struct Srv {
    devs: Vec<Box<dyn dev::Dev>>,
    chans: Vec<(DevId, ChanConfig)>,
}

fn adjust_chan_val(chan_cfg: &ChanConfig, val: f32) -> f32 {
    (chan_cfg.min +  (val as f64).powf(chan_cfg.exp) * (chan_cfg.max - chan_cfg.min)) as f32
}

impl<'a> Srv {
    pub fn new() -> Self {
        Srv {
            devs: Vec::new(),
            chans: Vec::new(),
        }
    }

    pub fn add_dev(&mut self, dev: Box<dyn dev::Dev>, chancfg: Option<Vec<ChanConfig>>)
                   -> DevId {

        let dev_id = DevId(self.devs.len() as u16);

        let chancfg = match chancfg {
            Some(chancfg) => {
                if chancfg.len() as u16 != dev.num_chans() {
                    panic!("invalid number of chans specified in chancfg: {} instead of {}",
                           chancfg.len(), dev.num_chans());
                }
                chancfg
            }
            None => (0..dev.num_chans()).map(|i| {
                let mut cc = ChanConfig::default();
                cc.index = i;
                cc
            }).collect()
        };

        dbg!(&chancfg);

        for chan in chancfg {
            self.chans.push((dev_id, chan))
        }
        self.devs.push(dev);

        dev_id
    }

    pub fn chans(&'a self) -> impl ExactSizeIterator<Item = (ChanId, String)> + 'a {
        self.chans.as_slice()
            .into_iter()
            .enumerate()
            .map(move |(chan_id, (dev_id, _dev_chan_id))| {
                let dev = self.get_dev(&dev_id);
                (ChanId(chan_id as u16),
                 format!("Chan {} {} \"{}\"", chan_id, dev_id, dev.name()))
            })
    }

    fn get_dev(&self, id: &DevId) -> &dyn dev::Dev {
        let DevId(idx) = id;
        self.devs[*idx as usize].as_ref()
    }

    pub fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            match val {
                Val::F32(fval) => {
                    // TODO error handling?
                    let chan = &self.chans[*cid as usize];
                    let chan_cfg = &chan.1;
                    let dev = &mut self.devs[chan.0.0 as usize];
                    dev.set_f32(chan_cfg.index, adjust_chan_val(chan_cfg, *fval))?;
                },
                _ => unimplemented!(),
            }
        }

        // sync all devs for now, optimize later
        for dev in self.devs.iter_mut() {
            let res = dev.as_mut().sync();
            match res {
                Err(e) => {
                    eprintln!("sync: {}", e);
                }
                Ok(_) => continue
            }
        }

        Ok(())
    }
}
