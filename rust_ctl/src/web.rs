
extern crate iron;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use crate::msg_handler::MsgHandler;
use crate::action::{Action, ChanSpec, ChanSpecGeneric};
use crate::config;
use std::sync::{Arc, RwLock};

struct Router<T: MsgHandler> {
    srv: Arc<RwLock<T>>,
    config: config::Config,
}

impl<T: MsgHandler> Router<T> {
    fn new(srv: Arc<RwLock<T>>, config: config::Config) -> Self {
        Router {
            srv: srv.clone(),
            config: config,
        }
    }
}

impl<T: 'static + MsgHandler> Handler for Router<T> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        // match self.routes.get(&req.url.path().join("/")) {
        //     Some(handler) => handler.handle(req),
        //     None => Ok(Response::with(status::NotFound))
        // }

        let path = req.url.path().join("/");

        println!("request to path: \"{}\"", path);

        match path.as_ref() {
            "" => {
                let resp = Response::with((
                    mime!(Text/Html),
                    status::Ok,
                    "
<body style=\"font-size: 72;\">

<a href=\"/off\">
  turn everything off
</a>

<br>
<br>

<a href=\"/on\">
  turn everything on
</a>

<br>
<br>
<br>

<a href=\"test\">
  test some new feature
</a>

</body>
"
                ));
                Ok(resp)
            }
            "off" => {
                let action = Action::Set(
                    ChanSpec::F32(
                        ChanSpecGeneric::<f32>::SomeWithDefault(0.0, vec![])
                    )
                );
                let srv = self.srv.clone();
                let result = action.perform(srv, &self.config);
                let resp = match result {
                    Ok(_) =>
                        Response::with((
                            mime!(Text/Html),
                            status::Ok,
                            "
<body style=\"font-size: 72;\">

seems like we turned everything off\n

<br><br>

<a href=\"/\">
  <- Back
</a>

</body>
"
                        )),
                    Err(e) =>
                        Response::with((
                            status::InternalServerError,
                            e
                        )),
                };

                Ok(resp)
            }
            "on" => {
                let action = Action::Set(
                    ChanSpec::F32(
                        ChanSpecGeneric::<f32>::SomeWithDefault(1.0, vec![])
                    )
                );
                let srv = self.srv.clone();
                let result = action.perform(srv, &self.config);
                let resp = match result {
                    Ok(_) =>
                        Response::with((
                            mime!(Text/Html),
                            status::Ok,
                            "
<body style=\"font-size: 72;\">

seems like we turned everything on\n

<br><br>

<a href=\"/\" style=\"font-size: 72;\">
  <- Back
</a>

</body>
"
                        )),
                    Err(e) =>
                        Response::with((
                            status::InternalServerError,
                            e
                        )),
                };

                Ok(resp)
            }
            "test" => {
                let resp = Response::with((
                    mime!(Text/Html),
                    status::Ok,
                    "
<body style=\"font-size: 72;\">

Eh?

<br><br>

<a href=\"/\" style=\"font-size: 72;\">
  <- Back
</a>

</body>
"
                ));
                Ok(resp)
            }
            _ => {
                Ok(Response::with(status::NotFound))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Web {
    pub listen_addr: String,
    config: config::Config,
}

const DEFAULT_LISTEN_ADDR: &str = "localhost:7373";

impl Web {
    pub fn new(listen_addr: Option<String>, config: config::Config) -> Result<Self, String> {
        let listen_addr = listen_addr.unwrap_or(DEFAULT_LISTEN_ADDR.to_string());
        Ok(Web {
            listen_addr: listen_addr,
            config: config
        })
    }

    pub fn run<T: 'static + MsgHandler>(&mut self,
                                        srv: Arc<RwLock<T>>)
                                        -> Result<(), String> {
        println!("listen_addr: {}", self.listen_addr);
        let router = Router::new(srv, self.config.clone());
        // unwrap?
        let listen_addr = self.listen_addr.clone();
        Iron::new(router).http(listen_addr).unwrap();
        Ok(())
        // unimplemented!();
    }
}
