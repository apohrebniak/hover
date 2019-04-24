extern crate hover;

use hover::common::{Address, Message, MessageType};
use hover::Hover;
use std::net::Ipv4Addr;
use std::ops::Deref;
use std::time::Duration;

fn main() {
    //create an instance of Hover
    //Node is created under the hood
    let mut hover = Hover::new(String::from("127.0.0.1"), 6202);

    //fully blocking start implementation.
    // Node is created and started to run in a separate thread
    hover.start();
    //
    //    hover.get_messaging_service().unwrap().send_to_address(
    //        String::from("Hello Hover!").into_bytes(),
    //        Address {
    //            ip: Ipv4Addr::LOCALHOST,
    //            port: 6203,
    //        },
    //    );
    //
    //    hover.add_msg_listener(|msg| println!("NEW INCOMING MESSAGE: {:?}", msg));

    hover
        .get_messaging_service()
        .unwrap()
        .send_to_address_receive(
            String::from("Hello Hover!").into_bytes(),
            Address {
                ip: Ipv4Addr::LOCALHOST,
                port: 6203,
            },
            Duration::new(10, 0),
        );

    //don't want to join on something
    //letf multicast and connection threads live on theis own
    std::thread::sleep_ms(600000);
}
