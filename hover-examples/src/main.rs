extern crate hover;

use hover::common::{Address, Message, MessageType};
use hover::service::messaging_service::MessagingService;
use hover::Hover;
use std::net::Ipv4Addr;

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
        Message {
            correlation: None,
            return_address: None,
            r#type: MessageType::REQUEST,
            payload: Vec::new(),
        },
        Address {
            ip: Ipv4Addr::LOCALHOST,
            port: 17021,
        },
    );

    hover.get_messaging_service().unwrap().broadcast(Message {
        correlation: None,
        r#type: MessageType::REQUEST,
        payload: Vec::new(),
        return_address: None,
    });

    //don't want to join on something
    //letf multicast and connection threads live on theis own
    std::thread::sleep_ms(600000);
}
