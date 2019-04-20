extern crate hover;

use hover::common::{Address, Message, MessageType};
use hover::Hover;
use std::net::Ipv4Addr;
use std::ops::Deref;

fn main() {
    //create an instance of Hover
    //Node is created under the hood
    let mut hover = Hover::new(String::from("127.0.0.1"), 6202);

    //fully blocking start implementation.
    // Node is created and started to run in a separate thread
    hover.start();

    hover.get_cluster_service();

    hover.get_messaging_service();

    hover.get_messaging_service().unwrap().send_to_address(
        String::from("Hello Hover!").into_bytes(),
        Address {
            ip: Ipv4Addr::LOCALHOST,
            port: 6203,
        },
    );

    match hover.get_cluster_service() {
        Ok(s) => {
            let l = s.read().unwrap().deref().get_members();
        }
        Err(_) => {}
    }

    hover.add_msg_listener(|msg| println!("NEW INCOMING MESSAGE: {:?}", msg));

    //don't want to join on something
    //letf multicast and connection threads live on theis own
    std::thread::sleep_ms(600000);
}
