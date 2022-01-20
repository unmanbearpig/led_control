use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum TaskMsg {
    // Stop doing what you're doing, but don't exit the thread
    Pause,

    // Does nothing
    Ping,
    Stop,
}

pub struct Task {
    pub name: String,
    pub chan: mpsc::Sender<TaskMsg>,
    pub join_handle: thread::JoinHandle<Result<(), String>>,
}

impl Task {
    pub fn is_running(&self) -> bool {
        let res = self.chan.send(TaskMsg::Ping);
        res.is_ok()
    }

    pub fn ask_to_pause(&mut self) {
        match self.chan.send(TaskMsg::Pause) {
            Ok(_) => {}
            Err(e) => {
                println!("got err while sending msg: {:?}", e);
            }
        }
    }

    pub fn ask_to_stop(&mut self) {
        match self.chan.send(TaskMsg::Stop) {
            Ok(_) => {}
            Err(e) => {
                println!("got err while sending msg: {:?}", e);
            }
        }
    }

    fn wait_for_stop(self) {
        while self.is_running() {
            thread::sleep(Duration::from_millis(1));
        }

        // FIXME:
        //  I only have a reference by join moves
        match self.join_handle.join() {
            Ok(res) => {
                if let Err(e) = res {
                    eprintln!("Task returned error: {}", e)
                }
            },
            Err(e) => {
                eprintln!(r#"
!!------------------------------------!!
    Error while joining thread
         I don't think I've seen it happen,
         should investigate!
Task name: {}
Error:
{:?}
!!------------------------------------!!
"#, self.name, e);
            }
        }
    }

    /// ask to stop and waits for it
    pub fn stop(mut self) {
        self.ask_to_stop();
        self.wait_for_stop();
    }
}
