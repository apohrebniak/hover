extern crate crossbeam_channel;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::common::{Address, Message, NodeMeta};
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub enum Event {
    Empty,
    DiscoveryIn { node_meta: NodeMeta },
    MessageIn { msg: Arc<Message> },
    MessageOut { msg: Arc<Message> },
}

pub struct EventLoop {
    atomic_run: Arc<AtomicBool>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    listeners: Arc<RwLock<Vec<Arc<RwLock<EventListener + Send + Sync>>>>>,
}

impl EventLoop {
    pub fn new() -> EventLoop {
        let (s, r): (Sender<Event>, Receiver<Event>) = crossbeam_channel::unbounded();

        EventLoop {
            atomic_run: Arc::new(AtomicBool::default()),
            sender: s,
            receiver: r,
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_listener(
        &mut self,
        listener: Arc<RwLock<EventListener + Send + Sync>>,
    ) -> Result<&mut EventLoop, Box<Error>> {
        self.listeners.write().unwrap().push(listener.clone());
        Ok((self))
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
                    let l_ = listeners_.read().unwrap();
                    for listener in l_.iter() {
                        listener.read().unwrap().on_event(event.clone());
                    }
                }
            }
        });
    }
}

pub trait EventListener {
    fn on_event(&self, event: Event);
}
