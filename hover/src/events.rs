extern crate crossbeam_channel;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::common::{Address, NodeMeta};
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub enum Event {
    Empty,
    DiscoveryEvent { node_meta: NodeMeta },
}

pub struct EventLoop {
    atomic_run: Arc<AtomicBool>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    listeners: Arc<Mutex<Vec<Arc<EventListener + Send + Sync>>>>,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        let (s, r): (Sender<Event>, Receiver<Event>) = crossbeam_channel::unbounded();

        EventLoop {
            atomic_run: Arc::new(AtomicBool::default()),
            sender: s,
            receiver: r,
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_listener(
        &mut self,
        listener: Arc<EventListener + Send + Sync>,
    ) -> Result<(), Box<Error>> {
        self.listeners.lock().unwrap().push(listener.clone());
        Ok(())
    }

    pub fn post_event(&self, event: Event) -> Result<(), Box<Error>> {
        match self.sender.send(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    //TODO: join on the thread
    pub fn start(&self) {
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

pub trait EventListener {
    fn on_event(&self, event: Event);
}
