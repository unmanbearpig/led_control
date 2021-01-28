#[allow(soft_unstable, dead_code, unused_variables)]

use std::mem;
use std::time::{Duration, SystemTime};
use std::slice;
use std::fmt;

// protocol for sending data via UDP / (or Unix sockets?)
// not intended for SPI or USB communication with the PWM controller

pub const MSG_MAX_VALS: usize = 61;
pub const MSG_VAL_SIZE: usize = 8;
pub const MSG_MAX_PAYLOAD: usize = MSG_VAL_SIZE * MSG_MAX_VALS;
pub const MSG_HEADER_SIZE: usize = 4 + 4 + 8 + 4 + 4;
pub const MSG_MAX_SIZE: usize = MSG_HEADER_SIZE + MSG_MAX_PAYLOAD;
pub const MSG_MAGIC: u8 = 0x1c;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChanId(pub u16);

impl fmt::Display for ChanId{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[chan {}]", self.0)
    }
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Val {
    U16(u16),
    F32(f32),
}

#[derive(Debug, PartialEq)]
pub enum SerErr {
    InvalidMagic,
    InvalidSize { num_vals: usize, expected_size: usize, actual_size: usize },
    InvalidTimestamp { s: u64, ns: u32 }
}

impl SerErr {
    pub fn is_invalid_size(&self) -> bool {
        match self {
            SerErr::InvalidSize {..} => true,
            _ => false
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ChanVal(pub ChanId, pub Val);

impl ChanVal {
    pub fn serialize_to_struct(&self, out: &mut ChanValSer) {
        out.chan_id = self.0.0;

        match self.1 {
            Val::U16(val) => {
                out.tag = 0;
                out.val[0..2].copy_from_slice(&val.to_le_bytes()[..]);
                out.val[2..].fill(0);
            }

            Val::F32(val) => {
                out.tag = 1;
                out.val.copy_from_slice(&val.to_le_bytes()[..]);
            }
        }
    }

    pub fn serialize_to_buf(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= mem::size_of::<ChanValSer>());

        let ser: &mut ChanValSer = unsafe {
            mem::transmute(buf.as_ptr())
        };
        self.serialize_to_struct(ser);
        8
    }

    pub fn deserialize_from_struct(ser: &ChanValSer) -> Result<Self, SerErr> {
        Ok(ChanVal(
            ChanId(ser.chan_id),
            match ser.tag {
                0 => {
                    let bytes = [ser.val[0], ser.val[1]];
                    Val::U16(u16::from_le_bytes(bytes))
                },
                1 => Val::F32(unsafe { mem::transmute(ser.val) }),
                _ => panic!("invalid ChanVal tag {}", ser.tag)
            }
        ))
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self, SerErr> {
        assert!(buf.len() >= mem::size_of::<ChanValSer>());
        let ser: &mut ChanValSer = unsafe {
            mem::transmute(buf.as_ptr())
        };

        ChanVal::deserialize_from_struct(ser)
    }
}

#[repr(C)]
pub struct ChanValSer {
    chan_id: u16,
    tag: u16, // just type so far
    val: [u8; 4],
}

impl Default for ChanValSer {
    fn default() -> Self {
        ChanValSer {
            chan_id: 0,
            tag: 0,
            val: [0, 0, 0, 0]
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Msg {
    pub seq_num: u16,
    pub timestamp: SystemTime,
    pub vals: Vec<ChanVal>,
}

impl Msg {
    pub fn serialize(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= MSG_MAX_SIZE);

        let ser: &mut  MsgHeaderSer = unsafe { mem::transmute(buf.as_ptr()) };

        ser.magic = MSG_MAGIC;
        ser.flags = 0;
        ser._reserved = 0;
        ser.seq_num = self.seq_num;

        let dur = self.timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        ser.timestamp_s = dur.as_secs();
        ser.timestamp_ns = dur.subsec_nanos();

        assert!(self.vals.len() <= MSG_MAX_VALS);
        ser.num_vals = self.vals.len() as u16;

        let data: &mut [ChanValSer] = unsafe {
            let data_ptr: *mut ChanValSer =
                buf.as_mut_ptr().offset(MSG_HEADER_SIZE as isize)
                as *mut ChanValSer;
            slice::from_raw_parts_mut(data_ptr, self.vals.len())
        };

        for (i, val) in self.vals.iter().enumerate() {
            val.serialize_to_struct(&mut data[i]);
        }

        // dbg!(MSG_HEADER_SIZE);
        // dbg!(self.vals.len());
        // dbg!(mem::size_of_val(&*data));

        // MSG_HEADER_SIZE + (self.vals.len() * MSG_VAL_SIZE)
        MSG_HEADER_SIZE + mem::size_of_val(&*data)
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self, SerErr> {
        let header: &MsgHeaderSer = unsafe { mem::transmute(buf.as_ptr()) };
        if header.magic != MSG_MAGIC {
            return Err(SerErr::InvalidMagic);
        }
        let expected_size =
            MSG_HEADER_SIZE + header.num_vals as usize * MSG_VAL_SIZE;
        // dbg!(header.num_vals);

        if expected_size != buf.len() {
            return Err(SerErr::InvalidSize {
                num_vals: header.num_vals as usize,
                expected_size: expected_size,
                actual_size: buf.len(),
            })
        }
        // ignoring flags and _reserved

        let vals: &[ChanValSer] = unsafe {
            let ptr = buf.as_ptr().offset(MSG_HEADER_SIZE as isize) as * const ChanValSer;
            slice::from_raw_parts(ptr, header.num_vals as usize)
        };

        let mut dur = Duration::from_secs(header.timestamp_s);
        dur += Duration::from_nanos(header.timestamp_ns as u64);
        let timestamp = SystemTime::UNIX_EPOCH.checked_add(dur);
        if timestamp.is_none() {
            return Err(SerErr::InvalidTimestamp {
                s: header.timestamp_s,
                ns: header.timestamp_ns,
            })
        }
        let timestamp = timestamp.unwrap();

        let out_vals: Vec<ChanVal> =
            vals.iter()
            .map( |v| ChanVal::deserialize_from_struct(v).into_iter() )
            .flatten()
            .collect();

        Ok(Msg {
            seq_num: header.seq_num,
            timestamp: timestamp,
            vals: out_vals,
        })
    }
}

// serialized Msg
#[repr(C)]
#[derive(Debug)]
pub struct MsgHeaderSer {
    magic: u8, // msg_magic = 1c
    flags: u8, // unused?
    _reserved: u16,

    seq_num: u16,
    num_vals: u16,

    timestamp_s: u64,
    timestamp_ns: u32,
    _reserved2: u32,
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;

    #[test]
    fn test_msg_ser_size() {
        assert_eq!(mem::size_of::<MsgHeaderSer>(), MSG_HEADER_SIZE as usize);
    }

    #[test]
    fn test_chan_val_ser() {
        assert_eq!(mem::size_of::<ChanValSer>(), 8 as usize);
    }

    #[test]
    fn test_chan_val_serialize() {
        let cv = ChanVal(ChanId(42), Val::U16(12345));
        let buf = &mut [0u8; 8];

        const EXPECTED_BYTES: [u8; 8] = [
            0x2a, 0x00,  // chan
            0x00, 0x00,  // flags
            0x39, 0x30,  // value
            0x00, 0x00,  // not used
        ];

        assert_eq!(cv.serialize_to_buf(buf), 8);
        assert_eq!(buf, &EXPECTED_BYTES);
    }

    #[test]
    fn test_chan_val_roundtrip() {
        let cv = ChanVal(ChanId(42), Val::U16(12345));
        let buf = &mut [0u8; 8];
        cv.serialize_to_buf(buf);
        let newcv = ChanVal::deserialize(buf).unwrap();

        assert_eq!(cv, newcv);
    }

    #[test]
    fn test_max_msg_size() {
        assert_eq!(512, MSG_MAX_SIZE);
    }

    #[test]
    fn test_msg_roundtrip() {
        let msg = Msg {
            seq_num: 12345,
            timestamp: SystemTime::now(),
            vals: vec!(ChanVal(ChanId(32), Val::U16(54))),
        };

        assert_eq!(512, MSG_MAX_SIZE);
        let buf = &mut [0u8; MSG_MAX_SIZE];
        assert_eq!(buf.len(), 512);
        let len = msg.serialize(buf);
        assert_eq!(
            MSG_HEADER_SIZE + MSG_VAL_SIZE, len,
            "msg serialize returns the number of used bytes");
        assert_eq!(msg, Msg::deserialize(&buf[0..len]).unwrap());
    }

    #[test]
    fn test_msg_deserialization() {
        let buf = &mut [0u8; MSG_MAX_SIZE];
        assert_eq!(SerErr::InvalidMagic,
                   Msg::deserialize(&buf[..]).unwrap_err());
        buf[0] = 0x1c;
        assert!(Msg::deserialize(&buf[..]).unwrap_err().is_invalid_size());
    }

    #[bench]
    #[allow(unused_must_use)]
    fn bench_msg_roundtrip(b: &mut test::Bencher) {
        b.iter(|| {
            let msg = Msg {
                seq_num: 12345,
                timestamp: SystemTime::now(),
                vals: vec!(ChanVal(ChanId(32), Val::U16(54))),
            };
            let buf = &mut [0u8; MSG_MAX_SIZE];
            let len = msg.serialize(buf);
            Msg::deserialize(&buf[0..len]);
        });
    }
}
