
use rusb;
// #[path = "old_proto.rs"] mod old_proto;
// #[path = "dev.rs"] mod dev;
use crate::dev::{self, Dev};
use std::time::Duration;
use std::fmt;

#[allow(dead_code)]
pub struct UsbDev {
    devhandle: rusb::DeviceHandle<rusb::GlobalContext>,
    bus_number: u8,
    dev_addr: u8,
    raw_vals: [u16; 3],
    _name: String,
}

impl fmt::Display for UsbDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl dev::Dev for UsbDev {
    fn name(&self) -> &String {
        &self._name
    }

    fn num_chans(&self) -> u16 {
        3
    }

    /// sets the internal state of the LED to the float value
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if val > 1.0 {
            return Err(format!(
                "UsbDev set_f32: value {} for chan {} is larger than 1.0",
                val, chan
            ))
        }

        if chan >= self.num_chans() {
            return Err(format!(
                "UsbDev set_f32: Invalid chan {}, only {} are available",
                chan, self.num_chans()))
        }

        let raw_val = (val * self.max_int() as f32).round() as u16;
        self.set_raw(chan, raw_val)
    }

    /// sends the set LED values to the device
    fn sync(&mut self) -> Result<(), String> {
        // eprintln!("usb write: {:?}", self.raw_vals);
        let endpoint = self.usb_endpoint();
        let timeout = self.timeout();
        // let data = self.raw_msg.into_slice();
        let data: &[u8; 6] = unsafe { std::mem::transmute(&self.raw_vals) };

        // print_bytes(data);

        let res = self.devhandle.write_interrupt(endpoint, data, timeout);
        match res {
            Ok(numbytes) => {
                if numbytes != data.len() {
                    eprintln!("USB sync: written {} of {} bytes", numbytes, data.len());
                }
                Ok(())
            }
            Err(e) => {
                Err(format!("USB sync error: {}", e))
            }
        }
    }



}

impl UsbDev {
    pub fn new(devhandle: rusb::DeviceHandle<rusb::GlobalContext>, bus_number: u8, dev_addr: u8) -> Self {
        let name = format!("USB Bus {} Dev {}", bus_number, dev_addr);

        UsbDev {
            devhandle: devhandle,
            bus_number: bus_number,
            dev_addr: dev_addr,
            raw_vals: [0u16; 3],
            _name: name,
        }
    }

    fn usb_endpoint(&self) -> u8 {
        5
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(6)
    }

    // not sure if needed
    pub fn max_int(&self) -> u16 {
        22126
    }

    /// doesn't scale the value i.e. doesn't take `max_int` into account
    pub fn set_raw(&mut self, chan: u16, val: u16) -> Result<(), String> {
        if chan >= self.num_chans() {
            return Err(format!(
                "UsbDev set_raw: Invalid chan {}, only {} are available",
                chan, self.num_chans()))
        }


        self.raw_vals[chan as usize] = val;
        Ok(())
    }

    pub fn find_devs() -> Result<Vec<Self>, String> {
        let devs = rusb::devices();

        let devs = match devs {
            Err(e) => {
                return Err(format!("USB device enumeration: {}", e))
            }
            Ok(d) => d
        };

        let devs = devs.iter().filter(|dev| {
            let desc = dev.device_descriptor().unwrap();
            desc.vendor_id() == 0xCAFE && desc.product_id() == 0xCAFE
        });

        let mut led_devs = Vec::new();
        for dev in devs {
            let handle = dev.open();
            match handle {
                Ok(h)  => led_devs.push(UsbDev::new(h, dev.bus_number(), dev.address())),
                Err(e) => return Err(format!("could not open dev {:?}: {}", dev, e))
            }
        }

        Ok(led_devs)
    }
}
