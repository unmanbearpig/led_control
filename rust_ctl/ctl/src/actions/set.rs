use leds::frame::Frame;
use leds::chan_spec::ChanSpec;
use proto;
use leds::dev::{DevWrite};
use leds::msg_handler::{MsgHandler};
use leds::chan_description::{ChanDescription, HasChanDescriptions};
use std::sync::{Arc, Mutex};

pub fn run_msg<T: MsgHandler>(chan_spec: &ChanSpec, srv: Arc<Mutex<T>>) -> Result<(), String> {
    let mut srv = srv.lock().map_err(|e| format!("{:?}", e))?;

    match chan_spec {
        ChanSpec::F32(spec) => {
            // need some ChanSpec(Generic?) method
            // that will give us the values for each specified chan
            let chan_descriptions: Vec<ChanDescription> = srv.chan_descriptions();
            let chanvals = spec.resolve_for_chans(chan_descriptions.as_slice())?;

            let chanvals = chanvals
                .into_iter()
                .map(|(cid, v)| proto::v1::ChanVal(proto::v1::ChanId(cid), proto::v1::Val::F32(v)))
                .collect();

            let msg = proto::v1::Msg::new(0, chanvals);

            srv.handle_msg(&msg)
        }
        ChanSpec::U16(_) => unimplemented!(),
    }
}

pub fn run_dev<T: DevWrite + HasChanDescriptions>(
    chan_spec: &ChanSpec, dev: Arc<Mutex<T>>
) -> Result<(), String> {
    let mut dev = dev.lock().map_err(|e| format!("{:?}", e))?;

    match chan_spec {
        ChanSpec::F32(spec) => {
            // need some ChanSpec(Generic?) method
            // that will give us the values for each specified chan
            let chan_descriptions: Vec<ChanDescription> =
                dev.chan_descriptions();

            let mut frame = Frame::empty();

            let chanvals =
                spec.resolve_for_chans(chan_descriptions.as_slice())?
                .into_iter()
                .map(|(cid, v)| (cid, v));
            for (cid, val) in chanvals {
                frame.set(cid, val);
            }
            dev.set_frame(&frame)
        }
        ChanSpec::U16(_) => unimplemented!(),
    }
}
