#![feature(test, div_duration)]
#![feature(iter_intersperse)]

#![allow(clippy::redundant_field_names)]

mod error;
mod action;
mod chan;
mod chan_spec;
mod config;
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
mod term_bar;
mod test_dev;
mod udp_srv;
mod udpv1_dev;
mod udpv2_dev;
mod usb;
mod wacom;
mod web;
mod action_spec;
mod actions;
mod init_devs;
mod frame;
mod chan_description;
mod tag;
mod template;
mod wrapper;

#[macro_use]
extern crate rust_embed;

use std::env;

fn main() -> Result<(), String> {
    let (action, config) = config::from_args(env::args())?;
    if let Some(action) = &action {
        action.init()?.perform(&config.configuration)?;
    }

    Ok(())
}
