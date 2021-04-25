extern crate tiny_http;
extern crate form_urlencoded;
use url::Url;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::fmt;

use std::io::Cursor;

use crate::actions;
use crate::chan_spec::{ChanSpec, ChanSpecGeneric};
use crate::config;
use crate::msg_handler::{MsgHandler};
use crate::chan_description::{ChanDescription, HasChanDescriptions};
use crate::dev::Dev;
use askama::Template;
use crate::tag::Tag;

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
    Err(&'a str), // TODO: clippy warning: should probably be used somewhere
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
#[template(path = "chan.html", escape = "none")]
struct ChanTemplate {
    chan: ChanDescription,
    value: f32,
}

#[derive(Template)]
#[template(path = "tag.html", escape = "none")]
struct TagTemplate<'a>(&'a Tag);

impl ChanTemplate {
    fn tags(&self) -> Vec<TagTemplate> {
        self.chan.tags.iter()
            .map(|tag| TagTemplate(tag))
            .collect()
    }
}

impl ChanTemplate {
    fn path(&self) -> String {
        format!("/chans/{}", self.chan.chan_id)
    }
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    msg: Option<FlashMsg<'a>>,
    chans: Vec<ChanTemplate>,
}

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:7373";

struct WebState<T: fmt::Debug> {
    base_url: Url,
    output: Arc<Mutex<T>>,

    #[allow(dead_code)]
    output_config: config::Config,

    http: tiny_http::Server,
    task: Option<Task>,
    smoother_slow: Arc<Mutex<MovingAverage<T>>>,
    smoother_fast: Arc<Mutex<MovingAverage<T>>>,
}

#[derive(Clone, Copy)]
enum SmoothingType {
    Slow, Fast
}

impl<T: 'static + Dev + HasChanDescriptions + fmt::Debug> WebState<T> {
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

    fn fade_all_to(&mut self, val: f32, ok_msg: &str) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            val,
            vec![],
        )), ok_msg, SmoothingType::Slow)
    }

    fn fade_to(&mut self, chan_spec: &ChanSpec, ok_msg: &str, smoothing: SmoothingType) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();
        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let smoother = match smoothing {
            SmoothingType::Slow => self.smoother_slow.clone(),
            SmoothingType::Fast => self.smoother_fast.clone(),
        };

        let join_handle = {
            let smoother = smoother.clone();
            thread::spawn(move || {
                Runner::run(smoother, rx)
            })
        };

        let mut msg = FlashMsg::Ok(ok_msg);

        let result = actions::set::run_dev(chan_spec, smoother);
        msg = msg.and_result(&result);

        if let Err(e) = &result {
            eprintln!("fade_all_to: action.perform error: {:?}", e);
        }

        self.task = Some(Task {
            name: "Smooth set val".to_string(),
            chan: tx,
            join_handle,
        });

        self.home_with(Some(msg))
    }

    fn on(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_all_to(
            1.0,
            "Is everything on?<br> <small>Kick Vanya if it's not!</small>",
        )
    }

    fn off(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_all_to(
            0.0,
            "Is everything off? <br> <small>Kick Vanya if it's not!</small>",
        )
    }

    fn disco(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();
        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let join_handle = {
            let output = self.output.clone();

            thread::spawn(move || demo::hello::run_with_channel(output, rx))
        };

        self.task = Some(Task {
            name: "Hello task from web test".to_string(),
            chan: tx,
            join_handle,
        });

        self.home_with(Some(FlashMsg::Ok("Wooooo111!!!")))
    }

    fn chans(&mut self) -> Vec<ChanTemplate> {
        let output = self.output.lock().unwrap();
        output
            .chan_descriptions()
            .into_iter()
            .map(|chan| {
                let value = output.get_f32(chan.chan_id).unwrap();
                ChanTemplate { chan, value }
            })
            .collect()
    }

    fn home_with(&mut self, msg: Option<FlashMsg>) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let template = HomeTemplate {
            msg,
            chans: self.chans(),
        };
        // todo fix unwrap
        let resp_str = template.render().unwrap();
        let data = resp_str.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(tiny_http::StatusCode(200), Vec::new(), cur, Some(len), None)
    }

    fn home(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.home_with(None)
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

        let extension: Option<&str> = path_segments.last()
            .map(|seg| seg.split('.').last())
            .flatten();

        const DEFAULT_CONTENT_TYPE: &str = "text/plain";
        let content_type: &str = match extension {
            Some("css") => "text/css",
            Some("js") => "text/javascript",
            Some(_) => DEFAULT_CONTENT_TYPE,
            None => DEFAULT_CONTENT_TYPE,
        };

        let content_type_header = tiny_http::Header::from_bytes(
            &b"Content-Type"[..], content_type)
            .unwrap();

        tiny_http::Response::new(
            tiny_http::StatusCode(200),
            vec![content_type_header],
            cur,
            Some(len),
            None)
    }

    fn handle_chans(
        &mut self,
        url: Url,
        req: &mut tiny_http::Request,
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
        match req.method() {
            tiny_http::Method::Post => {}
            m => return self.err404(m, url.to_string().as_ref()),
        }

        let mut path_segments = url.path_segments().unwrap();
        path_segments.next();
        let chan_id_str = match path_segments.next() {
            Some(chan_id_str) => chan_id_str,
            None => return self.err404(req.method(), url.to_string().as_ref()),
        };

        let (val, smoothing_type) = match path_segments.next() {
            Some("on") => (1.0, SmoothingType::Slow),
            Some("off") => (0.0, SmoothingType::Slow),
            Some("set") => {
                let mut body: Vec<u8> = Vec::new();
                req.as_reader().read_to_end(&mut body).unwrap();// TODO fix unwrap

                let mut value: Option<f32> = None;
                for (k, v) in form_urlencoded::parse(body.as_slice()) {
                    match k.as_ref() {
                        "value" => {
                            let f32_val: f32 = v.parse().unwrap(); // TODO fix unwrap
                            value = Some(f32_val);
                        }
                        other => {
                            // log and ignore
                            println!("unexpected form parameter {} with value '{}'", other, v);
                        }
                    }
                }

                match value {
                    Some(v) => (v, SmoothingType::Fast),
                    None => {
                        todo!()
                    }
                }
            }
            Some(_) => return self.err404(req.method(), url.to_string().as_ref()),
            None => return self.err404(req.method(), url.to_string().as_ref()),
        };

        let ok_msg = format!("chan {} set to {}", chan_id_str, val);

        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::Some(vec![(
            chan_id_str.to_string(),
            val,
        )])), ok_msg.as_ref(), smoothing_type)
    }

    /// All static files should start with /assets/
    fn handle_request(&mut self, mut req: tiny_http::Request) {
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
            (_, Some("chans")) => self.handle_chans(url, &mut req),
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

    pub fn run<T: 'static + MsgHandler + Dev>(
        &mut self,
        srv: Arc<Mutex<T>>,
        config: config::Config,
    ) -> Result<(), String> {
        let http = tiny_http::Server::http::<&str>(self.listen_addr.as_ref())
            .map_err(|e| format!("server err: {:?}", e))?;

        let ma_slow = MovingAverage::new(
            srv.clone(),
            Duration::from_millis(4),
            Duration::from_millis(900),
        );

        let ma_slow = Arc::new(Mutex::new(ma_slow));


        let ma_fast = MovingAverage::new(
            srv.clone(),
            Duration::from_millis(4),
            Duration::from_millis(50),
        );
        let ma_fast = Arc::new(Mutex::new(ma_fast));


        let mut server = WebState {
            base_url: Url::parse(format!("http://{}", self.listen_addr).as_ref()).unwrap(),
            output: srv,
            output_config: config,
            http,
            task: None,
            smoother_slow: ma_slow,
            smoother_fast: ma_fast,
        };

        server.run();

        Ok(())
    }
}
