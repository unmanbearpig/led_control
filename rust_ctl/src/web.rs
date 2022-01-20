extern crate tiny_http;
extern crate form_urlencoded;
use url::Url;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::fmt;

use std::io::{Write, Cursor};

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

use crate::demo::Fade;
use std::time::Duration;

use crate::runner::Runner;

// TODO we send messages to all devices even when setting only 1 channel
// TODO do we send messages to devices in parallel?

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

    fader: Arc<Mutex<Fade<T>>>,
    fade_task: Task,
}

#[derive(Clone, Copy)]
enum FadingType {
    Slow, Fast, TensOfMinutes
}

impl FadingType {
    fn duration(&self) -> Duration {
        match self {
            // TODO change me
            // Slow fade in
            FadingType::TensOfMinutes => Duration::from_secs(1800),

            // On / Off buttons
            FadingType::Slow => Duration::from_millis(900),

            // Fast transition on manual adjustments
            FadingType::Fast => Duration::from_millis(100),
        }
    }
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

    fn stop_fade(&mut self) {
        self.fade_task.ask_to_pause();
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

    fn fade_all_to(&mut self, val: f32, ok_msg: &str)
            -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            val,
            vec![],
        )), ok_msg, FadingType::Slow)
    }

    fn fade_to<S: AsRef<str>>(&mut self, chan_spec: &ChanSpec, ok_msg: S,
               fading: FadingType)
            -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_task();
        // let (tx, rx) = mpsc::channel::<TaskMsg>();

        // let fader = Arc::new(Mutex::new(Fade::new(
        //     self.output.clone(),
        //     Duration::from_millis(4),
        //     fading.duration(),
        // )));

        // let join_handle = {
        //     let fader = fader.clone();
        //     thread::spawn(move || {
        //         Runner::run(fader, rx)
        //     })
        // };

        // let mut msg = FlashMsg::Ok(ok_msg.as_ref());

        // let result = actions::set::run_dev(chan_spec, fader);
        // msg = msg.and_result(&result);

        // if let Err(e) = &result {
        //     eprintln!("fade_all_to: action.perform error: {:?}", e);
        // }

        // self.task = Some(Task {
        //     name: "Smooth set val".to_string(),
        //     chan: tx,
        //     join_handle,
        // });

        {
            let mut fader = self.fader.lock().unwrap();
            fader.fade_duration = fading.duration();
        }

        let mut msg = FlashMsg::Ok(ok_msg.as_ref());

        let result = actions::set::run_dev(chan_spec, self.fader.clone());
        msg = msg.and_result(&result);

        self.home_with(Some(msg))
    }

    fn on(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_all_to(
            1.0,
            "Is everything on?<br> <small>Kick Vanya if it's not!</small>",
        )
    }

    fn slow_fade_in(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let fade_type = FadingType::TensOfMinutes;
        let duration = fade_type.duration();
        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            1.0,
            vec![],
        )), format!("Doing slow fade in ({:?})", duration),
        fade_type)
    }

    fn slow_fade_out(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let fade_type = FadingType::TensOfMinutes;
        let duration = fade_type.duration();
        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::SomeWithDefault(
            0.0,
            vec![],
        )), format!("Doing slow fade out ({:?})", duration),
        fade_type)
    }

    fn off(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.fade_all_to(
            0.0,
            "Is everything off? <br> <small>Kick Vanya if it's not!</small>",
        )
    }

    fn disco(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_fade();
        self.stop_task();
        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let join_handle = {
            let output = self.output.clone();
            let mut disco_configs: Vec<demo::hello::DiscoChanConfig> =
                Vec::new();
            for dev in self.output_config.devs.iter() {
                if let Some(chans) = &dev.chans {
                    for _chan in chans.iter() {
                        let disco_config =
                            demo::hello::DiscoChanConfig::default();
                        disco_configs.push(disco_config);
                    }
                }
            }
            thread::spawn(move || demo::hello::run_with_config(
                    output, disco_configs, rx))
        };

        self.task = Some(Task {
            name: "Hello task from web test".to_string(),
            chan: tx,
            join_handle,
        });

        self.home_with(Some(FlashMsg::Ok("Wooooo111!!!")))
    }

    fn disco_harder(&mut self) -> tiny_http::Response<Cursor<Vec<u8>>> {
        self.stop_fade();
        self.stop_task();
        let (tx, rx) = mpsc::channel::<TaskMsg>();

        let join_handle = {
            let output = self.output.clone();
            let mut disco_configs: Vec<demo::hello::DiscoChanConfig> =
                Vec::new();
            for dev in self.output_config.devs.iter() {
                if let Some(chans) = &dev.chans {
                    for chan in chans.iter() {
                        let disco_config =
                            chan.disco_config.clone().unwrap_or_default();
                        disco_configs.push(disco_config);
                    }
                }
            }
            thread::spawn(move || demo::hello::run_with_config(
                    output, disco_configs, rx))
        };

        self.task = Some(Task {
            name: "Hello task from web test".to_string(),
            chan: tx,
            join_handle,
        });

        self.home_with(Some(FlashMsg::Ok("Enjoy the colors!")))
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

    fn home_with(&mut self, msg: Option<FlashMsg>) 
            -> tiny_http::Response<Cursor<Vec<u8>>> {
        let template = HomeTemplate {
            msg,
            chans: self.chans(),
        };
        // todo fix unwrap
        let resp_str = template.render().unwrap();
        let data = resp_str.into_bytes();
        let len = data.len();
        let cur = Cursor::new(data);
        tiny_http::Response::new(tiny_http::StatusCode(200), Vec::new(),
                                 cur, Some(len), None)
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

        tiny_http::Response::new(tiny_http::StatusCode(404), Vec::new(), cur,
                                 Some(len), None)
    }

    /// path_segments exclude the first segment ("/assets")
    fn handle_static_asset(&mut self, path_segments: Vec<&str>)
            -> tiny_http::Response<Cursor<Vec<u8>>> {
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

    /// Responds with json hashmap of chan_id: value
    fn handle_chans_json(
        &mut self,
        _url: Url,
        _req: &mut tiny_http::Request
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
        let chanvals: Vec<(u16, f32)> = {
            let output = self.output.lock().unwrap();
            output.chan_descriptions().into_iter()
            .map(|chan| (chan.chan_id, output.get_f32(chan.chan_id).unwrap()) )
            .collect()
        };
        let mut out: Vec<u8> = Vec::new();
        write!(out, "{{ ").unwrap();
        let mut chanvals = chanvals.into_iter().peekable();
        while let Some((cid, val)) = chanvals.next() {
            write!(out, "\"{cid}\": {val}").unwrap();
            if chanvals.peek().is_some() {
                write!(out, ", ").unwrap();
            }
        }
        write!(out, " }}").unwrap();

        let len = out.len();
        let cur = Cursor::new(out);
        tiny_http::Response::new(
            tiny_http::StatusCode(200),
            Vec::new(),
            cur,
            Some(len),
            None)
    }

    /// Sets channel values
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

        let (val, fading_type) = match path_segments.next() {
            Some("on") => (1.0, FadingType::Slow),
            Some("off") => (0.0, FadingType::Slow),
            Some("set") => {
                let mut body: Vec<u8> = Vec::new();
                // TODO fix unwrap
                req.as_reader().read_to_end(&mut body).unwrap();

                let mut value: Option<f32> = None;
                for (k, v) in form_urlencoded::parse(body.as_slice()) {
                    match k.as_ref() {
                        "value" => {
                            // TODO fix unwrap
                            let f32_val: f32 = v.parse().unwrap(); 
                            value = Some(f32_val);
                        }
                        other => {
                            // log and ignore
                            println!(
                                "unexpected form parameter {} with value '{}'",
                                other, v);
                        }
                    }
                }

                match value {
                    Some(v) => (v, FadingType::Fast),
                    None => {
                        todo!()
                    }
                }
            }
            Some(_) => return self.err404(req.method(), url.to_string()
                                          .as_ref()),
            None => return self.err404(req.method(), url.to_string().as_ref()),
        };

        let ok_msg = format!("chan {} set to {}", chan_id_str, val);

        self.fade_to(&ChanSpec::F32(ChanSpecGeneric::<f32>::Some(vec![(
            chan_id_str.to_string(),
            val,
        )])), ok_msg, fading_type)
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
            (tiny_http::Method::Get,  Some("chans.json")) =>
                self.handle_chans_json(url, &mut req),
            (tiny_http::Method::Post, Some("on")) => self.on(),
            (tiny_http::Method::Post, Some("off")) => self.off(),
            (tiny_http::Method::Post, Some("slow_fade_in")) =>
                self.slow_fade_in(),
            (tiny_http::Method::Post, Some("slow_fade_out")) =>
                self.slow_fade_out(),
            (tiny_http::Method::Post, Some("disco")) => self.disco(),
            (tiny_http::Method::Post, Some("disco_harder")) =>
                self.disco_harder(),
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
        let listen_addr = listen_addr
            .unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_string());

        Ok(Web { listen_addr })
    }

    pub fn run<T: 'static + MsgHandler + Dev>(
        &mut self,
        srv: Arc<Mutex<T>>,
        config: config::Config,
    ) -> Result<(), String> {
        let http = tiny_http::Server::http::<&str>(self.listen_addr.as_ref())
            .map_err(|e| format!("server err: {:?}", e))?;

        let fader = Arc::new(Mutex::new(Fade::new(
                        srv.clone(),
                        Duration::from_millis(6), Duration::from_millis(0))));

        let (fade_tx, fade_rx) = mpsc::channel::<TaskMsg>();
        let fade_join_handle = {
            let fader = fader.clone();
            thread::spawn(move || {
                Runner::run(fader, fade_rx)
            })
        };

        let fade_task = Task {
            name: "Fade task".to_string(),
            chan: fade_tx,
            join_handle: fade_join_handle,
        };

        let mut server = WebState {
            base_url: Url::parse(format!("http://{}", self.listen_addr)
                                 .as_ref()).unwrap(),
            output: srv,
            output_config: config,
            http,
            task: None,

            fader: fader,
            fade_task: fade_task,
        };

        server.run();

        Ok(())
    }
}
