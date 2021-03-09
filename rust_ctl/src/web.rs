
extern crate iron;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use crate::msg_handler::MsgHandler;
use crate::action::{Action, ChanSpec, ChanSpecGeneric};
use crate::config;
use crate::task::{TaskMsg, Task};
use crate::demo;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc;
use std::thread;


#[derive(Default)]
struct WebState {
    task: Option<Task>,
}

impl WebState {
    fn running_task(&mut self) -> Option<Task> {
        if self.task.is_none() {
            return None
        }

        let task = self.task.take().unwrap();
        if !task.is_running() {
            return None
        }

        Some(task)
    }
}

struct Router<T: MsgHandler> {
    srv: Arc<RwLock<T>>,
    config: config::Config,
    state: Arc<Mutex<WebState>>,
}

impl<T: MsgHandler> Router<T> {
    fn new(srv: Arc<RwLock<T>>, config: config::Config) -> Self {
        Router {
            srv: srv.clone(),
            config: config,
            state: Arc::new(Mutex::new(
                WebState { task: None }))
        }
    }
}

fn web_page(content: String) -> String {
    format!("
<body style=\"font-size: 72;\">
{}
</body>
", content)
}

fn web_msg(msg: &str) -> String {
    web_page(format!(
        "
{}
<br><br>
<a href=\"/\" style=\"font-size: 72;\">
  <- Back
</a>

", msg))
}

impl<T: 'static + MsgHandler> Router<T> {
    fn stop_task(&self) {
        let state = self.state.clone();
        let mut state = state.lock().unwrap();
        let mut task: Option<Task> = state.running_task();
        if task.is_some() {
            let task: Option<Task> = task.take();
            let task: Task = task.unwrap();
            task.stop();
            state.task = None;
        }
    }
}

const LINKS: &str = "
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
";

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
                let task_text = {
                    let state = self.state.clone();
                    {
                        let state = state.lock().unwrap();
                        match &state.task {
                            None => "no task running".to_string(),
                            Some(task) => {
                                let task_name: &str = task.name.as_ref();
                                format!(
                                    "task {} is running",
                                    task_name)
                            }
                        }
                    }
                };
                let resp = Response::with((
                    mime!(Text/Html),
                    status::Ok,
                    web_page(format!("{}<br>\n{}", task_text, LINKS)),
                ));
                Ok(resp)
            }
            "off" => {
                self.stop_task();
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
                            web_msg(
                                "seems like we turned everything off"
                            )
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
                self.stop_task();

                println!("task should be stopped");

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
                            web_msg(
                                "seems like we turned everything on"
                            )
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
                self.stop_task();
                let state = self.state.clone();
                let mut state = state.lock().unwrap();

                let (tx, rx) = mpsc::channel::<TaskMsg>();

                let join_handle = {
                    let srv = self.srv.clone();
                    thread::spawn(move || {
                        demo::hello::run_with_channel(srv, rx)
                    })
                };

                state.task = Some(Task {
                    name: "Hello task from web test".to_string(),
                    chan: tx,
                    join_handle: join_handle,
                });

                let resp = Response::with((
                    mime!(Text/Html),
                    status::Ok,
                    web_msg("Eh?")));
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
