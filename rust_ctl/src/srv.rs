use crate::dev;
use crate::proto;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevId(u16);

impl Display for DevId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[Dev {}]", self.0)
    }
}

pub struct Srv<'a> {
    devs: Vec<&'a mut dyn dev::Dev>,
    chans: Vec<(DevId, u16)>,
}

impl<'a> Srv<'a> {
    pub fn new() -> Self {
        Srv {
            devs: Vec::new(),
            chans: Vec::new(),
        }
    }

    pub fn add_dev<T: dev::Dev>(&mut self, dev: &'a mut T) -> DevId {
        let dev_id = DevId(self.devs.len() as u16);
        for chan in 0..dev.num_chans() {
            self.chans.push((dev_id, chan))
        }
        self.devs.push(dev);

        dev_id
    }

    pub fn chans(&self) -> Vec<(u16, String)> {
        self.chans.iter().enumerate().map(|(chan_id, (dev_id, dev_chan_id))| {
            let dev = self.get_dev(dev_id);
            (chan_id as u16,
             format!("Chan {} {} \"{}\"", chan_id, dev_id, dev.name()))
        }).collect()
    }

    fn get_dev(&self, id: &DevId) -> &dyn dev::Dev {
        let DevId(idx) = id;
        self.devs[*idx as usize]
    }

    pub fn handle_msg(&mut self, msg: proto::Msg) -> Result<(), String> {
        unimplemented!();
    }

    fn handle_chan_val(&mut self, chan_val: proto::ChanVal) -> Result<(), String> {
        unimplemented!();
    }
}
