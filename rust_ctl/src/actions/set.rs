use crate::chan_spec::ChanSpec;
use crate::proto;
use std::sync::{Arc, Mutex};
use crate::msg_handler::{ChanDescription, MsgHandler};

pub fn run(chan_spec: &ChanSpec, srv: Arc<Mutex<dyn MsgHandler>>) -> Result<(), String> {
    let mut srv = srv.lock().map_err(|e| format!("{:?}", e))?;

    match chan_spec {
        ChanSpec::F32(spec) => {
            // need some ChanSpec(Generic?) method
            // that will give us the values for each specified chan
            let chan_descriptions: Vec<ChanDescription> = srv.chan_descriptions();
            let chanvals = spec.resolve_for_chans(chan_descriptions.as_slice())?;

            let chanvals = chanvals
                .into_iter()
                .map(|(cid, v)| proto::ChanVal(proto::ChanId(cid), proto::Val::F32(v)))
                .collect();

            let msg = proto::Msg::new(0, chanvals);

            srv.handle_msg(&msg)
        }
        ChanSpec::U16(_) => unimplemented!(),
    }
}
