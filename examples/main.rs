extern crate hover;

use bincode::{deserialize, serialize};
use hover::common::{Address, Message, MessageType};
use hover::events::{Event, EventListener};
use hover::Hover;
use std::net::Ipv4Addr;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

struct Foo {}

impl EventListener for Foo {
    fn on_event(&self, event: Event) {
        println!("Hello event!");
    }
}

fn main() {
    //create an instance of Hover
    //Node is created under the hood
    let mut hover = Arc::new(RwLock::new(Hover::default().unwrap()));

    //fully blocking start implementation.
    // Node is created and started to run in a separate thread
    hover.write().unwrap().start();

    let r: u8 = rand::random();
    let value = Arc::new(RwLock::new(r as f32));

    hover
        .write()
        .unwrap()
        .get_messaging_service()
        .unwrap()
        .read()
        .unwrap()
        .broadcast(serialize(&*value.read().unwrap()).unwrap());

    let value_ = value.clone();
    let hover_ = hover.clone();
    hover.write().unwrap().add_broadcast_listener(move |msg| {
        let in_: f32 = deserialize(msg.payload.as_slice()).unwrap();
        let current = value_.read().unwrap().clone();

        if in_ < current {
            hover_
                .read()
                .unwrap()
                .get_messaging_service()
                .unwrap()
                .read()
                .unwrap()
                .broadcast(serialize(&current).unwrap());
        }

        *value_.write().unwrap() = current.max(in_);
    });

    loop {
        println!("----MAX VALUE={}", value.read().unwrap());

        std::thread::sleep_ms(500);
    }
}
