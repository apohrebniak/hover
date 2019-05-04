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
    let mut hover = Arc::new(RwLock::new(Hover::new(String::from("127.0.0.1"), 6202)));

    //fully blocking start implementation.
    // Node is created and started to run in a separate thread
    hover.write().unwrap().start();

    let value = Arc::new(RwLock::new(42_f32));

    hover
        .write()
        .unwrap()
        .get_messaging_service()
        .unwrap()
        .read()
        .unwrap()
        .broadcast(serialize(&*value.read().unwrap()).unwrap());

    let v_ = value.clone();
    hover.write().unwrap().add_broadcast_listener(move |msg| {
        let in_: f32 = deserialize(msg.payload.as_slice()).unwrap();
        let l = v_.read().unwrap().clone();
        *v_.write().unwrap() = (l + in_) / 2_f32;
    });

    loop {
        println!("----AVERAGE VALUE={}", value.read().unwrap());

        std::thread::sleep_ms(500);
    }
}
