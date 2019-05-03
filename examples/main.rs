extern crate hover;

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

    let ms = hover.write().unwrap().get_messaging_service().unwrap();
    ms.read()
        .unwrap()
        .broadcast(String::from("Hello broadcast!").into_bytes());

    //    let foo = Foo {};
    //
    //    hover.write().unwrap().add_event_listener(foo);

    //        loop {
    //            hover
    //                .read().unwrap()
    //                .get_messaging_service()
    //                .unwrap()
    //                .send_to_address_receive(
    //                    String::from("Hello Hover!").into_bytes(),
    //                    Address {
    //                        ip: Ipv4Addr::LOCALHOST,
    //                        port: 6203,
    //                    },
    //                    Duration::new(10, 0),
    //                ).map(|msg| println!("REPLIED: {:?}", msg)).unwrap();
    //            std::thread::sleep_ms(3000);
    //        }
    //
    //    let hover_ = hover.clone();
    //
    //    hover.write().unwrap().add_msg_listener(move |msg| {
    //        match &msg.clone().return_address {
    //            Some(addr) => {
    //                hover_.read().unwrap().get_messaging_service().unwrap().reply(
    //                    msg.corId.clone(),
    //                    String::from("Bye Hover!").into_bytes(),
    //                    Address {
    //                        ip: addr.ip,
    //                        port: addr.port,
    //                    },
    //                );
    //            }
    //            _ => {}
    //        }
    //    });

    //don't want to join on something
    //letf multicast and connection threads live on theis own
    std::thread::sleep_ms(600000);
}
