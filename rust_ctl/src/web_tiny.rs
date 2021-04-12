
extern crate tiny_http;
use url::{Url};

use std::sync::{Arc, RwLock};
use std::sync::mpsc;

use std::io::Cursor;

use crate::msg_handler::MsgHandler;
use crate::action::Action;
use crate::chan_spec::{ChanSpec, ChanSpecGeneric};
use crate::config;

pub struct Web {
    pub listen_addr: String,
}

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:7373";

struct WebState<T: MsgHandler> {
    base_url: Url,
    output: Arc<RwLock<T>>,
    output_config: config::Config,
    http: tiny_http::Server,
}

impl<T: 'static + MsgHandler> WebState<T> {
    pub fn run(&mut self) {
        loop {
            println!("recving...");
            let res = self.http.recv();
            let req = match res {
                Ok(req) => req,
                Err(e) => {
                    println!("http error: {:?}", e);
                    continue;
                }
            };

            self.handle_request(req);
        }
    }

    fn parse_relative_url(&self, url: &str) -> Result<Url, url::ParseError> {
        self.base_url.join(url)
    }

    fn req_relative_url(&self, req: tiny_http::Request) -> Result<Url, String> {
        let url = req.url();
        let url = match self.parse_relative_url(url) {
            Ok(url) => url,
            Err(err) => {
                return Err(format!("url parse error: {:?}", err));
            }
        };
        unimplemented!();
    }

    fn string_resp(code: u16, body: String) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let data = body.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(
            tiny_http::StatusCode(code),
            Vec::new(),
            cur,
            Some(len),
            None
        )

    }

    fn on(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        // todo: stop_task
        let action = Action::Set(
            ChanSpec::F32(
                ChanSpecGeneric::<f32>::SomeWithDefault(1.0, vec![])
            )
        );
        let srv = self.output.clone();
        let result = action.perform(srv, &self.output_config);

        println!("/on not implemented");
        let data = "hello".to_string().into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        let resp = tiny_http::Response::new(
            tiny_http::StatusCode(200),
            Vec::new(),
            cur,
            Some(len),
            None
        );

        resp
    }

    fn off(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        // todo: stop task
        let action = Action::Set(
            ChanSpec::F32(
                ChanSpecGeneric::<f32>::SomeWithDefault(0.0, vec![])
            )
        );
        let srv = self.output.clone();
        let result = action.perform(srv, &self.output_config);

        println!("/off not implemented");
        unimplemented!()
    }

    fn home(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let resp_str = "home".to_string();
        let data = resp_str.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(
            tiny_http::StatusCode(200),
            Vec::new(),
            cur,
            Some(len),
            None
        )
    }

    fn err404(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        println!("error 404 not implemented");
        unimplemented!()
    }

    /// path_segments exclude the first segment ("/assets")
    fn handle_static_asset(&mut self, path_segments: Vec<&str>) -> tiny_http::Response<Cursor<Vec<u8>>> {
        println!("static asset: path_segments: {:?}", path_segments);

        unimplemented!()
    }

    fn handle_request(&mut self, req: tiny_http::Request) {
        let url = req.url();
        let url = match self.parse_relative_url(url) {
            Ok(url) => url,
            Err(err) => {
                println!("url parse error: {:?}", err);
                return
            }
        };

        let mut path_segments = url.path_segments().unwrap();

        let first_segment = path_segments.next();
        println!("first_segment = {:?}", first_segment);

        let resp = match first_segment {
            Some("on") => {
                self.on()
            }
            Some("off") => {
                self.off()
            }
            Some("") => { // why doesn't work?
                self.home()
            }
            Some("assets") => {
                // TODO: check it's a GET request
                // TODO: HEAD request
                self.handle_static_asset(path_segments.collect())
            }
            Some(_) => {
                self.err404()
            }
            None => {
                // root?
                self.home()
            }
        };

        req.respond(resp).unwrap();
    }
}

impl Web {
    pub fn new(listen_addr: Option<String>) -> Result<Self, String> {
        let listen_addr = listen_addr.unwrap_or(
            DEFAULT_LISTEN_ADDR.to_string());

        Ok(Web {
            listen_addr: listen_addr,
        })
    }

    pub fn run<T: 'static + MsgHandler>(&mut self, srv: Arc<RwLock<T>>, config: config::Config)
                                        -> Result<(), String> {

        let http = tiny_http::Server::http::<&str>(self.listen_addr.as_ref())
            .map_err(|e| {
                format!("server err: {:?}", e)
            })?;

        let mut server = WebState {
            base_url: Url::parse(
                format!("http://{}", self.listen_addr).as_ref()).unwrap(),
            output: srv,
            output_config: config,
            http: http,
        };

        server.run();

        // outdated
        // loop {
        //     let res = http.recv();
        // }

        Ok(())
    }
}
