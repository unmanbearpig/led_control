use crate::proto::{Val, ChanVal, ChanId, Msg};

use std::ops::{Add};

#[derive(Debug, Clone, PartialEq)]
pub struct Frame<T: Clone> {
    pub vals: Vec<Option<T>>,
}

use std::collections::VecDeque; // TODO use iterator
impl Frame<f32> {
    pub fn simple_average(frames: &VecDeque<Frame<f32>>) -> Frame<f32> {
        let mut result = Frame::new(0);
        let mut counts: Frame<usize> = Frame::new(0);

        for frame in frames.iter() {
            result.ensure_bounds(frame.len() -1);
            result.add_assign(frame);
            for (chan, _) in frame.iter_some() {
                counts.add_to_val(chan, 1);
            }
        }

        for (cid, v) in result.iter_mut_some() {
            *v = *v / counts.get(cid).unwrap() as f32;
        }
        result
    }

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
        for (cid, v) in self.iter_some() {
            msg.vals[cid as usize].1 = Val::F32(*v)
        }
    }

}

impl<T: Clone + PartialEq> Frame<T> {
    pub fn is_subset_of(&self, other: &Frame<T>) -> bool {
        self.vals.iter().zip(other.vals.iter()).all(|(a, b)| {
            match a {
                None => true,
                Some(_) => a == b,
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

    pub fn len(&self) -> u16 {
        self.vals.len() as u16
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
    where T: Add<Output = T> + Copy
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

    // pub fn add_to_val_ref(&mut self, chan: u16, val: &T)
    // where T: AddAssign
    // {
    //     self.ensure_bounds(chan);
    //     let prev_val: &mut Option<T> = self.vals.get_mut(chan as usize).unwrap();
    //     if prev_val.is_some() {
    //         prev_val.iter_mut()
    //             .map(|prev: &mut T| prev.add_assign(val.clone()));
    //     } else {
    //         *prev_val = Some(val.clone());
    //     }
    // }

    pub fn add_assign(&mut self, other: &Self)
    where T: Add<Output = T> + Copy
    {
        for (chan, val) in other.iter_some() {
            self.add_to_val(chan, *val);
        }
    }

    pub fn iter_some(&self) -> impl Iterator<Item = (u16, &T)> + '_ {
        self.vals.iter().enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(cid, v)| (cid as u16, v.as_ref().unwrap()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u16, &mut Option<T>)> + '_ {
        self.vals.iter_mut().enumerate()
            .map(|(cid, v)| (cid as u16, v))
    }

    pub fn iter_mut_some(&mut self) -> impl Iterator<Item = (u16, &mut T)> + '_ {
        self.iter_mut()
            .filter(|(cid, v)| v.is_some())
            .map(|(cid, v)| (cid, v.iter_mut().next().unwrap()))
    }

    // pub fn iter_mut_some(&mut self) -> impl Iterator<Item = &mut T> + '_ {
    //     self.vals.iter_mut()
    //         .filter(|v| v.is_some())
    //         .map(|v| v.as_mut().unwrap())
    // }
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

    #[test]
    fn test_avg_frame() {
        let mut frames: VecDeque<Frame<f32>> = VecDeque::new();
        frames.push_back(Frame { vals: vec![None,      None, Some(0.1), None] });
        frames.push_back(Frame { vals: vec![Some(0.3), None, Some(0.9), None] });
        frames.push_back(Frame { vals: vec![Some(0.1) ] });

        let avg: Frame<f32> = Frame::simple_average(&frames);

        assert_eq!(avg, Frame { vals: vec![Some(0.2), None, Some(0.5), None] })
    }
}
