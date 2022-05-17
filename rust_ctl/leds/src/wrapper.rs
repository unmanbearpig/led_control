use proto::v1::ChanId;
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use std::sync::{Arc, Mutex};
use crate::dev::{DevNumChans};

pub trait Wrapper {
    type Output;
    fn output(&self) -> Arc<Mutex<Self::Output>>;
}

impl<W: Wrapper> HasChanDescriptions for W where
    W::Output: HasChanDescriptions
{
    fn chans(&self) -> Vec<(ChanId, String)> {
        let output = self.output();
        let output = output.lock().unwrap();
        output.chans()
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        let output = self.output();
        let output = output.lock().unwrap();
        output.chan_descriptions()
    }
}

impl<W: Wrapper> DevNumChans for W where
    W::Output: DevNumChans
{
    fn num_chans(&self) -> u16 {
        let output = self.output();
        let output = output.lock().unwrap();
        output.num_chans()
    }
}
