

pub struct ValRangeF32 {
    start: u16,
    end:   u16,
    vals: &[f32]
}

pub struct AddrDesc {
    start: u16,
    len: u16,
    mask: u64,
}

#[repr(u8)]
enum MsgType {
    Ping,
    PingResp,
    DataRead,
    DataReadResp,
    DataWrite,
}

pub struct MsgHeader {
    len: u16,
    typ: MsgType,
}

enum Msg {
    Ping,
    PingResp,
    DataRead
}

// fn deserialize(bytes: &'a u8)
