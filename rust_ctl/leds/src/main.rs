#![feature(test, div_duration)]
#![feature(iter_intersperse)]

#![allow(clippy::redundant_field_names)]

#![allow(dead_code)]

mod error;
mod action;
mod chan;
mod chan_spec;
mod configuration;
mod parse_ip_port;
mod controller;
mod coord;
mod cuboid;
mod demo;
mod dev;
mod dev_stats;
mod filters;
mod msg_handler;
mod runner;
mod srv;
mod task;
mod test_dev;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod usb;
mod wacom;
mod init_devs;
mod frame;
mod chan_description;
mod tag;
mod template;
mod wrapper;

fn main() -> Result<(), String> {
    Err("Use ctl binary instead".to_string())
}
