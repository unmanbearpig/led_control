use serde_derive::{Serialize, Deserialize};
use std::num::ParseIntError;
use std::collections::BTreeMap;

use crate::msg_handler::ChanDescription;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChanSpecGeneric<F> {
    Each(Vec<F>),
    // (chan_id/tag, value)
    Some(Vec<(String, F)>),
    SomeWithDefault(F, Vec<(String, F)>)
}

fn find_chans<'a>(chan_descr: &'a str, chans: &'a [ChanDescription]) -> impl Iterator<Item = u16> + 'a {
    let maybe_cid: Result<u16, ParseIntError> = chan_descr.parse();
    chans.iter().filter(move |cdesc| {
        let maybe_cid = maybe_cid.clone();
        if maybe_cid.is_ok() {
            cdesc.chan_id == maybe_cid.unwrap()
        } else {
            cdesc.tags.iter()
                .find(|tag| {
                    let t: &str = tag.as_ref();
                    t == chan_descr
                }).is_some()
        }
    }).map(|cdesc| cdesc.chan_id)
}

impl<F> ChanSpecGeneric<F> {
    // chans: (id, (tags or something))
    pub fn resolve_for_chans(&self, chans: &[ChanDescription]) -> Result<Vec<(u16, F)>, String>
    where F: Copy
    {
        match self {
            ChanSpecGeneric::Each(vals) => {
                if chans.len() != vals.len() {
                    return Err(format!(
                        "Provided {} vals, but we have {} channels",
                        vals.len(), chans.len()))
                }

                Ok(chans.iter().zip(vals).map(|(cdesc, val)| (cdesc.chan_id, *val)).collect())
            }
            ChanSpecGeneric::Some(chanvals) => {
                let mut val_map: BTreeMap<u16, F> = BTreeMap::new();

                for (chan_descr, val) in chanvals.iter() {
                    let mut found_some = false;
                    for chan_id in find_chans(chan_descr, chans) {
                        val_map.insert(chan_id, *val);
                        found_some = true;
                    }
                    if !found_some {
                        return Err(format!("could not find channels for '{}'", chan_descr))
                    }
                }

                Ok(val_map.into_iter().collect())
            }
            ChanSpecGeneric::SomeWithDefault(default, chanvals) => {
                let mut val_map: BTreeMap<u16, F> = BTreeMap::new();

                for cid in chans.iter().map(|cdesc| cdesc.chan_id) {
                    val_map.insert(cid, *default);
                }

                for (chan_descr, val) in chanvals.iter() {
                    let mut found_some = false;
                    for chan_id in find_chans(chan_descr, chans) {
                        val_map.insert(chan_id, *val);
                        found_some = true;
                    }
                    if !found_some {
                        return Err(format!("could not find channels for '{}'", chan_descr))
                    }
                }

                Ok(val_map.into_iter().collect())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChanSpec {
    F32(ChanSpecGeneric<f32>),
    U16(ChanSpecGeneric<u16>),
}

impl ChanSpec {
    /// examples:
    ///   0.8         => SomeWithDefault(0.8, vec![])
    ///   1,.7        => Each(vec![1.0, 0.7])
    ///   0:0.6,3:1.0 => Some((0, 0.6), (3, 1.0))
    ///   .1,1:.7     => SomeWithDefault(0.1, vec![(1, 0.7)])
    // TODO: how to make it generic for f32 and u16?
    pub fn parse_f32(string: &str) -> Result<ChanSpec, String> {
        let parsed_parts: Vec<(Option<&str>, f32)> = string.split(",").map(|p| {
            // there must be a better way than Vec
            let chanval: Vec<&str> = p.split(":").collect();
            let chanval: (Option<&str>, f32) = match chanval.len() {
                0 => Err(format!("Invalid blank argument in {}", string)),
                1 => Ok((None, chanval[0].parse().map_err(|e| format!("{:?}", e))?)),
                2 => {
                    Ok((
                        Some(chanval[0]),
                        chanval[1].parse().map_err(|e| format!("{:?}", e))?
                    ))
                },
                _ => Err(format!("Invalid argument '{}', expected CHAN:VAL or VAL", p)),
            }?;
            Ok(chanval)
        }).collect::<Result<Vec<(Option<&str>, f32)>, String>>()?;

        match parsed_parts.len() {
            0 => Err("No arguments provided".to_string()),
            1 => {
                let (chan, val) = parsed_parts[0];
                if chan.is_some() {
                    Ok(ChanSpec::F32(ChanSpecGeneric::Some(
                        vec![(chan.unwrap().to_string(), val)]
                    )))
                } else {
                    Ok(ChanSpec::F32(ChanSpecGeneric::SomeWithDefault(val, Vec::new())))
                }
            },
            _ => { // we know we have at least 2 items
                if parsed_parts[0].0.is_some() {
                    // must be Some
                    if !parsed_parts.iter().all(|(chan, _val)| chan.is_some()) {
                        return Err("If the first value specifies chan_id\
                                    then all of them should".to_string())
                    }

                    return Ok(ChanSpec::F32(
                        ChanSpecGeneric::Some(
                            parsed_parts.iter()
                                .map(|(chan, val)| (chan.unwrap().to_string(), *val))
                                .collect()
                        )
                    ))
                }

                // if we got here then it means the first item
                //   doesn't have chan specified

                if parsed_parts[1].0.is_some() { // SomeWithDefault
                    if !parsed_parts.iter().skip(1).all(|(chan, _val)| chan.is_some()) {
                        return Err("If the second value specifies chan_id\
                                    then all the rest of them should".to_string())
                    }

                    let (_, default_val) = parsed_parts[0];
                    return Ok(ChanSpec::F32(
                        ChanSpecGeneric::SomeWithDefault(
                            default_val,
                            parsed_parts.iter().skip(1)
                                .map(|(chan, val)| (chan.unwrap().to_string(), *val))
                                .collect()
                        )
                    ))
                }

                // if we got here then it must be Each
                if !parsed_parts.iter().all(|(chan, _val)| chan.is_none()) {
                    return Err("If the first 2 values don't specify chan \
                                then none of them should".to_string())
                }

                return Ok(ChanSpec::F32(
                    ChanSpecGeneric::Each(
                        parsed_parts.iter().map(|(_, val)| *val).collect()
                    )
                ))
            },
        }
    }

    pub fn parse_u16(_string: &str) -> Result<ChanSpec, String> {
        unimplemented!();
    }
}
