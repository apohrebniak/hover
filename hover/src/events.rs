extern crate crossbeam_channel;

use core::borrow::BorrowMut;
use std::error::Error;
use std::sync::{Arc, Mutex};

use self::crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone)]
pub enum Event {
    Empty,
}

struct EventLoop {
    atomic_run: Arc<AtomicBool>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    listeners: Arc<Mutex<Vec<Arc<EventListener + Send + Sync>>>>,
}

impl EventLoop {
    fn new() -> EventLoop {
        let (s, r): (Sender<Event>, Receiver<Event>) = crossbeam_channel::unbounded();

        EventLoop {
            atomic_run: Arc::new(AtomicBool::default()),
            sender: s,
            receiver: r,
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_listener(
        &mut self,
        listener: Arc<EventListener + Send + Sync>,
    ) -> Result<(), Box<Error>> {
        self.listeners.lock().unwrap().push(listener.clone());
        Ok(())
    }

    fn post_event(&self, event: Event) -> Result<(), Box<Error>> {
        match self.sender.send(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    //TODO: join on the thread
    fn start(&self) {
        self.atomic_run.store(true, Ordering::Relaxed);

        let running_ = self.atomic_run.clone();
        let receiver_ = self.receiver.clone();
        let listeners_ = self.listeners.clone();

        std::thread::spawn(move || {
            while running_.load(Ordering::Relaxed) {
                for event in receiver_.iter() {
                    let l_ = listeners_.lock().unwrap();
                    for listener in l_.iter() {
                        listener.on_event(event.clone());
                    }
                }
            }
        });
    }
}

trait EventListener {
    fn on_event(&self, event: Event);
}
