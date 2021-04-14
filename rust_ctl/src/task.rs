
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum TaskMsg {
    Ping, // does nothing
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

    pub fn ask_to_stop(&mut self) {
        println!("asking task to stop...");
        match self.chan.send(TaskMsg::Stop) {
            Ok(_) => {},
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
        let res = self.join_handle.join();
        println!("join task result: {:?}", res);
    }

    /// ask to stop and waits for it
    pub fn stop(mut self) {
        self.ask_to_stop();
        self.wait_for_stop();
    }
}
