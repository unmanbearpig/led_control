use crate::dev::{Dev, DevNumChans, DevRead, DevWrite};
use std::fmt;
use std::time::Duration;

pub struct UsbDev {
    devhandle: rusb::DeviceHandle<rusb::GlobalContext>,
    bus_number: u8,
    dev_addr: u8,
    raw_vals: [u16; 3],
    last_f32_vals: [f32; 3], // the values actually written to the device
    f32_vals: [f32; 3], // buffer that is not `sync`ed yet
}

impl fmt::Display for UsbDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "USB Bus {} Dev {}", self.bus_number, self.dev_addr)
    }
}

impl DevNumChans for UsbDev {
    fn num_chans(&self) -> u16 {
        3
    }
}

impl DevRead for UsbDev {
    fn get_f32(&self, chan: u16) -> Result<f32, String> {
        if chan > 2 {
            return Err(format!("chan {} out of bounds (0-2)", chan));
        }

        Ok(self.last_f32_vals[chan as usize])
    }
}

impl DevWrite for UsbDev {
    /// sets the internal state of the LED to the float value
    fn set_f32(&mut self, chan: u16, val: f32) -> Result<(), String> {
        if val > 1.0 {
            return Err(format!(
                "UsbDev set_f32: value {} for chan {} is larger than 1.0",
                val, chan
            ));
        }

        if chan >= self.num_chans() {
            return Err(format!(
                "UsbDev set_f32: Invalid chan {}, only {} are available",
                chan,
                self.num_chans()
            ));
        }

        self.f32_vals[chan as usize] = val;

        let raw_val = (val * self.max_int() as f32).round() as u16;
        self.set_raw(chan, raw_val)
    }

    /// sends the set LED values to the device
    fn sync(&mut self) -> Result<(), String> {
        // eprintln!("usb write: {:?}", self.raw_vals);
        let endpoint = self.usb_endpoint();
        let timeout = self.timeout();
        let data: &[u8; 6] = unsafe { &*(&self.raw_vals as *const [u16; 3] as *const [u8; 6]) };

        self.last_f32_vals = self.f32_vals;

        let res = self.devhandle.write_interrupt(endpoint, data, timeout);
        match res {
            Ok(numbytes) => {
                if numbytes != data.len() {
                    eprintln!("USB sync: written {} of {} bytes", numbytes, data.len());
                }
                Ok(())
            }
            Err(e) => Err(format!("USB sync error: {}", e)),
        }
    }
}

impl Dev for UsbDev {}

impl UsbDev {
    pub fn new(
        devhandle: rusb::DeviceHandle<rusb::GlobalContext>,
        bus_number: u8,
        dev_addr: u8,
    ) -> Self {
        UsbDev {
            devhandle,
            bus_number,
            dev_addr,
            raw_vals: [0u16; 3],
            f32_vals: [0.0; 3],
            last_f32_vals: [0.0; 3],
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
                chan,
                self.num_chans()
            ));
        }

        self.raw_vals[chan as usize] = val;
        Ok(())
    }

    pub fn find_devs() -> Result<Vec<Self>, String> {
        let devs = rusb::devices();

        let devs = match devs {
            Err(e) => return Err(format!("USB device enumeration: {}", e)),
            Ok(d) => d,
        };

        let devs = devs.iter().filter(|dev| {
            let desc = dev.device_descriptor().unwrap();
            desc.vendor_id() == 0xCAFE && desc.product_id() == 0xCAFE
        });

        let mut led_devs = Vec::new();
        for dev in devs {
            let handle = dev.open();
            match handle {
                Ok(h) => led_devs.push(UsbDev::new(h, dev.bus_number(), dev.address())),
                Err(e) => return Err(format!("could not open dev {:?}: {}", dev, e)),
            }
        }

        Ok(led_devs)
    }
}
