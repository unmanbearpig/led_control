use crate::proto::{Val, ChanVal, ChanId, Msg};

#[derive(Debug)]
pub struct Frame<T: Clone> {
    pub vals: Vec<Option<T>>,
}

impl Frame<f32> {
    /// Replaces frame values with values from msg
    ///   useful because msg might not contain values for all channels
    /// Keeps old values as is
    fn merge_msg(&mut self, msg: &Msg) {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            let val = match val {
                Val::U16(_) => unimplemented!(),
                Val::F32(val) => val,
            };

            self.vals[*cid as usize] = Some(*val);
        }
    }
}

impl<T: Clone> Frame<T> {
    pub fn new(num_chans: u16) -> Self {
        Frame::<T> {
            vals: vec![None; num_chans as usize]
        }
    }

    pub fn clear(&mut self) {
        for val in self.vals.iter_mut() {
            *val = None;
        }
    }

    pub fn set(&mut self, chan: u16, val: T) {
        let chan = chan as usize;
        if chan >= self.vals.len() {
            self.vals.resize_with(chan +1, Default::default);
        }
        self.vals[chan] = Some(val)
    }
}

#[cfg(test)]
mod frame_test {
    extern crate test;
    use super::*;

    #[test]
    fn test_new() {
        let frame: Frame<f32> = Frame::new(2);

        assert_eq!(frame.vals.len(), 2);
        assert_eq!(frame.vals[0], None);
        assert_eq!(frame.vals[1], None);
    }

    #[test]
    fn test_set_f32() {
        let mut frame = Frame::new(2);
        frame.set(1, 0.3);
        assert_eq!(frame.vals[1], Some(0.3));
    }

    #[test]
    fn test_set_f32_out_of_initial_bounds() {
        let mut frame = Frame::new(2);
        frame.set(15, 0.3);
        assert_eq!(frame.vals[15], Some(0.3));
    }

    #[test]
    fn test_clear() {
        let mut frame = Frame::new(2);
        frame.set(1, 0.3);
        frame.clear();
        assert_eq!(frame.vals[1], None);
    }

}
