extern crate tiny_http;
use url::Url;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use std::io::Cursor;

use crate::action::Action;
use crate::chan_spec::{ChanSpec, ChanSpecGeneric};
use crate::config;
use crate::msg_handler::MsgHandler;
use askama::Template;

use crate::demo;
use crate::task::{Task, TaskMsg};
use std::thread;

use crate::filters::moving_average::MovingAverage;
use std::time::Duration;

use crate::runner::Runner;

#[derive(RustEmbed)]
#[folder = "assets"]
struct StaticAsset;

pub struct Web {
    pub listen_addr: String,
}

#[derive(Template)]
#[template(path = "flash_msg.html", escape = "none")]
enum FlashMsg<'a> {
    Ok(&'a str),
    Err(&'a str),
}

impl<'a> FlashMsg<'a> {
    fn from_result<O>(result: &'a Result<O, String>, ok_msg: &'a str) -> Self {
        match result {
            Ok(_) => FlashMsg::Ok(ok_msg),
            Err(err_msg) => FlashMsg::Err(err_msg.as_ref()),
        }
    }

    fn and_result<O>(self, result: &'a Result<O, String>) -> FlashMsg {
        match self {
            FlashMsg::Ok(ok_msg) => FlashMsg::from_result(result, ok_msg),
            FlashMsg::Err(_) => self,
        }
    }
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    msg: Option<FlashMsg<'a>>,
}

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:7373";

struct WebState {
    base_url: Url,
    output: Arc<Mutex<dyn MsgHandler>>,
    output_config: config::Config,
    http: tiny_http::Server,
    task: Option<Task>,
    smoother: Arc<Mutex<MovingAverage>>,
}

impl WebState {
    fn running_task(&mut self) -> Option<Task> {
        self.task.as_ref()?;

        let task = self.task.take().unwrap();
        if !task.is_running() {
            return None;
        }

        Some(task)
    }

    fn stop_task(&mut self) {
        let mut task: Option<Task> = self.running_task();
        if task.is_some() {
            let task: Option<Task> = task.take();
            let task: Task = task.unwrap();
            task.stop();
            self.task = None;
        }
    }

    pub fn run(&mut self) {
        loop {
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

    fn on(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();
        let action = Action::Set(ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            1.0,
            vec![],
        )));
        let srv = self.smoother.clone();
        let result = action.perform(srv, &self.output_config);

        let flash_msg =
            FlashMsg::Ok("Is everything on?<br> <small>Kick Vanya if it's not!</small>")
                .and_result(&result);

        self.home_with(HomeTemplate {
            msg: Some(flash_msg),
        })
    }

    fn off(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();
        let action = Action::Set(ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            0.0,
            vec![],
        )));
        let srv = self.smoother.clone();
        let result = action.perform(srv, &self.output_config);

        let flash_msg =
            FlashMsg::Ok("Is everything off? <br> <small>Kick Vanya if it's not!</small>")
                .and_result(&result);

        self.home_with(HomeTemplate {
            msg: Some(flash_msg),
        })
    }

    fn disco(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();

        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let join_handle = {
            let output = self.output.clone();

            thread::spawn(move || demo::hello::run_with_channel(output.clone(), rx))
        };

        self.task = Some(Task {
            name: "Hello task from web test".to_string(),
            chan: tx,
            join_handle,
        });

        self.home_with(HomeTemplate {
            msg: Some(FlashMsg::Ok("Wooooo111!!!")),
        })
    }

    fn home_with(&mut self, template: HomeTemplate) -> tiny_http::Response<Cursor<Vec<u8>>> {
        // todo fix unwrap
        let resp_str = template.render().unwrap();
        let data = resp_str.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(tiny_http::StatusCode(200), Vec::new(), cur, Some(len), None)
    }

    fn home(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let template = HomeTemplate { msg: None };

        self.home_with(template)
    }

    fn err404(
        &mut self,
        method: &tiny_http::Method,
        path: &str,
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let resp_str = format!("404: Not found {} {}\n", method, path);
        let data = resp_str.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);

        tiny_http::Response::new(tiny_http::StatusCode(404), Vec::new(), cur, Some(len), None)
    }

    /// path_segments exclude the first segment ("/assets")
    fn handle_static_asset(
        &mut self,
        path_segments: Vec<&str>,
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let path: String = path_segments.join("/");
        let asset = StaticAsset::get(path.as_ref());

        let data: Vec<u8> = match asset {
            Some(content) => content.to_vec(),
            None => return self.err404(&tiny_http::Method::Get, path.as_ref()),
        };

        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(tiny_http::StatusCode(200), Vec::new(), cur, Some(len), None)
    }

    /// All static files should start with /assets/
    fn handle_request(&mut self, req: tiny_http::Request) {
        let url = req.url();
        let url = match self.parse_relative_url(url) {
            Ok(url) => url,
            Err(err) => {
                println!("url parse error: {:?}", err);
                return;
            }
        };

        let mut path_segments = url.path_segments().unwrap();

        let first_segment = path_segments.next();
        let resp = match (req.method(), first_segment) {
            (tiny_http::Method::Post, Some("on")) => self.on(),
            (tiny_http::Method::Post, Some("off")) => self.off(),
            (tiny_http::Method::Post, Some("disco")) => self.disco(),
            (tiny_http::Method::Get, Some("")) => self.home(),
            (tiny_http::Method::Get, Some("assets")) => {
                // TODO: check it's a GET request
                // TODO: HEAD request
                self.handle_static_asset(path_segments.collect())
            }
            (m, Some(_)) => self.err404(m, url.to_string().as_ref()),
            (_, None) => {
                // root?
                self.home()
            }
        };

        req.respond(resp).unwrap();
    }
}

impl Web {
    pub fn new(listen_addr: Option<String>) -> Result<Self, String> {
        let listen_addr = listen_addr.unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_string());

        Ok(Web { listen_addr })
    }

    pub fn run(
        &mut self,
        srv: Arc<Mutex<dyn MsgHandler>>,
        config: config::Config,
    ) -> Result<(), String> {
        let http = tiny_http::Server::http::<&str>(self.listen_addr.as_ref())
            .map_err(|e| format!("server err: {:?}", e))?;

        let ma = MovingAverage::new(
            srv.clone(),
            Duration::from_millis(4),
            Duration::from_millis(900),
        );

        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let ma = Arc::new(Mutex::new(ma));

        let join_handle = {
            let ma = ma.clone();
            // let srv = sync_dev.clone();
            thread::spawn(move || Runner::run(ma, rx))
        };

        let _task = Some(Task {
            name: "Hello task from web test".to_string(),
            chan: tx,
            join_handle,
        });

        let mut server = WebState {
            base_url: Url::parse(format!("http://{}", self.listen_addr).as_ref()).unwrap(),
            output: srv,
            output_config: config,
            http,
            task: None,
            smoother: ma,
        };

        server.run();

        Ok(())
    }
}
