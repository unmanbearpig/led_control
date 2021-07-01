#![feature(test, div_duration)]

mod action;
mod chan;
mod chan_spec;
mod config;
mod controller;
mod coord;
mod cuboid;
mod demo;
mod dev;
mod dev_stats;
mod filters;
mod msg_handler;
mod old_proto;
mod proto;
mod runner;
mod srv;
mod task;
mod term_bar;
mod test_dev;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod usb;
mod wacom;
mod web_tiny;
mod actions;
mod init_devs;
mod frame;
mod chan_description;
mod tag;

#[macro_use]
extern crate rust_embed;

use std::env;

fn main() -> Result<(), String> {
    let config = config::Config::from_args(env::args())?;
    if let Some(action) = &config.action {
        action.perform(&config)?;
    }

    Ok(())
}
