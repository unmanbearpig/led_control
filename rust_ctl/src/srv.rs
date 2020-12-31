use crate::dev;
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
    chans: Vec<(DevId, u16)>,
}

impl Srv {
    pub fn new() -> Self {
        Srv {
            devs: Vec::new(),
            chans: Vec::new(),
        }
    }

    pub fn add_dev(&mut self, dev: Box<dyn dev::Dev>) -> DevId {
        let dev_id = DevId(self.devs.len() as u16);
        for chan in 0..dev.num_chans() {
            self.chans.push((dev_id, chan))
        }
        self.devs.push(dev);

        dev_id
    }

    pub fn chans(&self) -> Vec<(ChanId, String)> {
        self.chans.iter().enumerate().map(|(chan_id, (dev_id, _dev_chan_id))| {
            let dev = self.get_dev(dev_id);
            (ChanId(chan_id as u16),
             format!("Chan {} {} \"{}\"", chan_id, dev_id, dev.name()))
        }).collect()
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
                    let chan = self.chans[*cid as usize];
                    let dev = &mut self.devs[chan.0.0 as usize];
                    dev.set_f32(chan.1, *fval)?;
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

    // fn handle_chan_val(&mut self, _chan_val: proto::ChanVal) -> Result<(), String> {
    //     unimplemented!();
    // }
}
