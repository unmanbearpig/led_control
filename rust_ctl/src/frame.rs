use crate::proto::{Val, ChanVal, ChanId, Msg};

use std::ops;

#[derive(Debug, Clone, PartialEq)]
pub struct Frame<T: Clone> {
    pub vals: Vec<Option<T>>,
}

impl Frame<f32> {
    /// Replaces frame values with values from msg
    ///   useful because msg might not contain values for all channels
    /// Keeps old values as is
    pub fn merge_msg(&mut self, msg: &Msg) {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            let val = match val {
                Val::U16(_) => unimplemented!(),
                Val::F32(val) => val,
            };

            self.vals[*cid as usize] = Some(*val);
        }
    }

    /// writes values to provided msg
    pub fn to_msg(&self, msg: &mut Msg) {
        for (cid, v) in self.iter_with_chans() {
            msg.vals[cid as usize].1 = Val::F32(*v)
        }
    }
}

impl<T: Clone + PartialEq> Frame<T> {
    pub fn is_subset_of(&self, other: &Frame<T>) -> bool {
        self.vals.iter().zip(other.iter()).all(|(a, b)| {
            match a {
                None => true,
                Some(a) => a == b,
            }
        })
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

    pub fn get(&self, chan: u16) -> Option<T>
        where T: Copy
    {
        self.vals.get(chan as usize).copied().flatten()
    }

    pub fn set(&mut self, chan: u16, val: T) {
        self.ensure_bounds(chan);
        self.vals[chan as usize] = Some(val)
    }

    /// makes sure the vals size is at least `len`
    fn ensure_bounds(&mut self, len: u16) {
        if len as usize >= self.vals.len() {
            self.vals.resize_with(len as usize +1, Default::default);
        }
    }

    pub fn add_to_val(&mut self, chan: u16, val: T)
        where T: ops::Add<Output = T> + Copy
    {
        self.ensure_bounds(chan);
        let prev_val = self.vals.get_mut(chan as usize).unwrap();
        *prev_val = Some(match prev_val {
            Some(prev_val) => {
                *prev_val + val
            }
            None => val,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.vals.iter()
            .filter(|v| v.is_some())
            .map(|v| v.as_ref().unwrap())
    }

    pub fn iter_with_chans(&self) -> impl Iterator<Item = (u16, &T)> + '_ {
        self.vals.iter().enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(cid, v)| (cid as u16, v.as_ref().unwrap()))
    }


    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.vals.iter_mut()
            .filter(|v| v.is_some())
            .map(|v| v.as_mut().unwrap())
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
