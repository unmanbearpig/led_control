
const LED_MSG_MAGIC: u16 = 0x1324;
const PAD_MSG16_LEN: usize = 13;
const PAD_MSGF32_LEN: usize = 6;
#[derive(Debug)]
#[repr(C)]
pub struct LedMsg16 {
    magic: u16,
    msgtype: u16,
    flags: u16,
    pad: [u8; 3],
    pub values: [u16; 4],
    reserved: [u8; PAD_MSG16_LEN],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LedMsgF32 {
    magic: u16,
    msgtype: u16,
    flags: u16,
    amount: f32,
    //pad: [u8; 6],
    pub values: [f32; 4],
    reserved: [u8; PAD_MSGF32_LEN],
}

impl Default for LedMsgF32 {
    fn default() -> Self {
        LedMsgF32 {
            magic: LED_MSG_MAGIC,
            msgtype: 2,
            flags: 1, // means the values are floats
            amount: 0.88888,
            //pad: [0; 6],
            values: [0.1, 1.0, 0.5, 0.9],
            reserved: [0; PAD_MSGF32_LEN],
        }
    }
}

impl<'a> LedMsgF32 {
    pub fn into_slice(&'a mut self) -> &'a mut [u8] {
        // am I violating aliasing rules here?
        unsafe {
            std::slice::from_raw_parts_mut(
                self as *mut LedMsgF32 as *mut u8,
                std::mem::size_of::<Self>())
        }
    }
}


impl<'a> LedMsg16 {
    pub fn into_slice(&'a mut self) -> &'a mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self as *mut LedMsg16 as *mut u8,
                std::mem::size_of::<Self>())
        }
    }
}

impl Default for LedMsg16 {
    fn default() -> Self {
        LedMsg16 {
            magic: LED_MSG_MAGIC,
            msgtype: 2, // #define LED_WRITE 2
            flags: 0, // 1 means the values are float
            pad: [11, 22, 33],
            values: [0x3333, 0xffff, 0x9999, 0xffff],
            reserved: [0; PAD_MSG16_LEN],
        }
    }
}

const NUM_VALUES: usize = 4;
