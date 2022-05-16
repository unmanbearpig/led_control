#![feature(test, div_duration)]
#![feature(iter_intersperse)]

mod error;
mod action;
mod wrapper;
mod producer;
mod actions;
mod action_spec;
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
mod init_devs;
mod frame;
mod chan_description;
mod tag;
mod template;
mod web;

#[macro_use]
extern crate rust_embed;

