use proto::v1::{Val, ChanVal, ChanId, Msg};

use std::ops::{Add};

#[derive(Debug, Clone, PartialEq)]
pub struct Frame<T: Clone> {
    pub vals: Vec<Option<T>>,
}

use std::collections::VecDeque; // TODO use iterator
impl Frame<f32> {
    pub fn empty() -> Self {
        Frame { vals: Vec::new() }
    }

    #[allow(unused)]
    pub fn simple_average(frames: &VecDeque<Frame<f32>>) -> Frame<f32> {
        let mut result = Frame::new(0);
        let mut counts: Frame<usize> = Frame::new(0);

        for frame in frames.iter() {
            result.ensure_bounds(frame.num_chans() -1);
            result.add_assign(frame);
            for (chan, _) in frame.iter_some() {
                counts.add_to_val(chan, 1);
            }
        }

        for (cid, v) in result.iter_mut_some() {
            *v /= counts.get(cid).unwrap() as f32;
        }
        result
    }

    /// Replaces frame values with values from msg
    ///   useful because msg might not contain values for all channels
    /// Keeps old values as is
    #[allow(unused)]
    pub fn merge_msg(&mut self, msg: &Msg) {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            let val = match val {
                Val::U16(_) => unimplemented!(),
                Val::F32(val) => val,
            };

            self.vals[*cid as usize] = Some(*val);
        }
    }

    #[allow(unused)]
    pub fn almost_same_as(&self, other: &Frame<f32>, margin: f32) -> bool {
        if self.vals.len() != other.vals.len() {
            return false
        }

        for (a, b) in self.vals.iter().zip(other.vals.iter()) {
            if a == b {
                continue;
            }

            if a.is_some() != b.is_some() {
                return false;
            }

            if a.is_none() {
                continue;
            }

            let a = a.unwrap();
            let b = b.unwrap();

            if !(a >= b - margin && a <= b + margin) {
                return false;
            }
        }
        true
    }

    pub fn print_vals(&self) {
        let conf = term_bar::config().print_val_digits(4);
        for val in self.vals.iter() {
            match val {
                Some(val) => conf.val(*val).print(),
                None => println!(),
            }
        }
    }
}

impl<T: Clone + PartialEq> Frame<T> {
    #[allow(unused)]
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

    pub fn num_chans(&self) -> u16 {
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

    pub fn set_all(&mut self, val: T) {
        for chan in 0..self.num_chans() {
            self.set(chan, val.clone())
        }
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

    pub fn iter_mut(&mut self)
            -> impl Iterator<Item = (u16, &mut Option<T>)> + '_ {
        self.vals.iter_mut().enumerate()
            .map(|(cid, v)| (cid as u16, v))
    }

    pub fn iter_mut_some(&mut self) -> impl Iterator<Item = (u16, &mut T)> + '_ {
        self.iter_mut()
            .filter(|(_, v)| v.is_some())
            .map(|(cid, v)| (cid, v.iter_mut().next().unwrap()))
    }

    pub fn merge_frame(&mut self, from: &Frame<T>) -> Result<(), String> {
        if from.num_chans() > self.num_chans() {
            return Err(format!(
                    "Cannot merge more frame channels than we have. \
We have {}, trying to merge {} channels",
                    self.num_chans(), from.num_chans()));
        }

        for (ii, val) in from.vals.iter().enumerate() {
            if val.is_some() {
                self.vals[ii] = val.clone();
            }
        }

        Ok(())
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

    #[test]
    fn test_avg_frame() {
        let mut frames: VecDeque<Frame<f32>> = VecDeque::new();
        frames.push_back(Frame { vals: vec![None,      None, Some(0.1), None] });
        frames.push_back(Frame { vals: vec![Some(0.3), None, Some(0.9), None] });
        frames.push_back(Frame { vals: vec![Some(0.1) ] });

        let avg: Frame<f32> = Frame::simple_average(&frames);

        assert_eq!(avg, Frame { vals: vec![Some(0.2), None, Some(0.5), None] })
    }

    #[test]
    fn test_is_subset_of() {
        let sup = Frame { vals: vec![Some(0.5), Some(0.1)] };
        let sub = Frame { vals: vec![None,      Some(0.1)] };
        assert_eq!(true, sub.is_subset_of(&sup));
        assert_eq!(false, sup.is_subset_of(&sub));
    }
}
