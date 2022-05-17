// temoprary while working on it
#![allow(dead_code)]

#[derive(Clone, Copy)]
pub struct ChanRange {
    start: u16,
    count: u16,
}

pub struct ValRangeF32<'a> {
    range: ChanRange,
    vals:  &'a [f32]
}

#[repr(u8)]
enum MsgType {
    Ping                = 0,
    PingResp            = 1,
    DataReadF32         = 2,
    DataReadResponseF32 = 3,
    DataWriteF32        = 4,
    GetConf //?
}

pub struct MsgHeader {
    len:  u16,
    typ:  MsgType,
}

enum Msg<'a> {
    Ping(u16),
    PingResp(u16),
    DataReadF32(ChanRange),
    DataReadResponseF32(ChanRange, &'a [f32]),
    DataWriteF32(ChanRange, &'a [f32]),
}

fn deserialize<'a>(bytes: &'a u8) -> Result<Msg, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    use super::*;

    #[test]
    fn test_serialization() {
    }

    #[bench]
    fn bench_serialization(b: &mut Bencher) {
    }
}
