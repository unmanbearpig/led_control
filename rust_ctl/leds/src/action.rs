use std::net::IpAddr;
use std::time::Duration;

use crate::actions;
use crate::chan_spec::ChanSpec;
use crate::chan_description::HasChanDescriptions;
use crate::config;
use crate::coord::Coord;
use crate::demo;
use crate::udp_srv;
use crate::web;
use crate::srv::Srv;
use serde_derive::{Deserialize, Serialize};

#[cfg(test)]
mod chan_spec_parse_test {
    use crate::chan_spec::{ChanSpec, ChanSpecGeneric};

    #[test]
    fn parse_all() {
        assert_eq!(
            ChanSpec::parse_f32(".4").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::SomeWithDefault(0.4, Vec::new()))
        )
    }

    #[test]
    fn parse_some_with_default() {
        assert_eq!(
            ChanSpec::parse_f32(".4,3:1.0").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::SomeWithDefault(
                0.4,
                vec![("3".to_string(), 1.0)]
            ))
        )
    }

    #[test]
    fn parse_some_1_arg() {
        assert_eq!(
            ChanSpec::parse_f32("1:.4").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Some(vec![("1".to_string(), 0.4)]))
        )
    }

    #[test]
    fn parse_some() {
        assert_eq!(
            ChanSpec::parse_f32("0:.4,2:.7").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Some(vec![
                ("0".to_string(), 0.4),
                ("2".to_string(), 0.7)
            ]))
        )
    }

    #[test]
    fn parse_each() {
        assert_eq!(
            ChanSpec::parse_f32(".4,.7").unwrap(),
            ChanSpec::F32(ChanSpecGeneric::Each(vec![0.4, 0.7]))
        )
    }
}

pub trait Action<'a>: std::fmt::Debug {
    fn perform(&self, config: &config::Config) -> Result<(), String>;
}
