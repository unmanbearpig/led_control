use std::fs::File;
use std::io::Read;

// packet format
// 1b ??
// 1b flags
// 2b xlow xhigh
// 2b ylow yhigh
// 1b pressure
// 1b is touching
// 1b distance
//

// flag bits:
//   0 nearby
//   1 receives xy location
//   2 receives xy, height and buttons
//   3 ??
//   4 0: normal end, 1: eraser end
//   5 up btn
//   6 down btn
//   7 is touching

#[repr(C, packed)]
#[derive(Default)]
pub struct WacomPacket {
    unknown1: u8,
    pub flags: u8,
    pub x: u16,
    pub y: u16,
    pub pressure: u8,
    pub is_touching: u8,
    pub distance: u8,
    unknown2: u8,
}

pub struct Wacom<'a> {
    #[allow(dead_code)]
    filename: &'a str,
    file: File,
}

impl<'a> Wacom<'a> {
    pub fn new(filename: &'a str) -> Result<Self, String> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => return Err(format!("{:?}", e)),
        };

        Ok(Wacom { filename, file })
    }

    pub fn read(&mut self, out: &mut WacomPacket) -> Result<(), String> {
        let buf: &mut [u8] =
            unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(out), 10) };
        let len = match self.file.read(buf) {
            Ok(l) => l,
            Err(e) => return Err(format!("{:?}", e)),
        };

        if len != 10 {
            return Err(format!("invalid length {} instead of 10", len));
        }

        Ok(())
    }
}

// fn main() -> io::Result<()> {
//     println!("Hello, world!");

//     let mut args = env::args();
//     args.next(); // skip executable name
//     let name = match args.next() {
//         Some(n) => n,
//         None => {
//             println!("expected name argument");
//             process::exit(1);
//        }
//     };

//     println!("name: {}", name);

//     let mut file = File::open(name)?;
//     let mut buf = vec![0u8; 10];
//     loop {
//         let numbytes = file.read(buf.as_mut())?;
//         let wacom: &WacomPacket = unsafe {
//             std::mem::transmute(buf.as_ptr())
//         };
//         println!("got {} bytes: {:?}", numbytes, buf);
//         println!("wacom: {:?}", wacom);
//     }
// }
