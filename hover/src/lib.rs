mod service;

use std::net::*;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::service::{ConnectionService, MulticastService, RunnableService};

pub struct Hover {
    host: Ipv4Addr,
    port: u16,
    node: Option<Node>,
    started: bool,
}

impl Hover {
    pub fn new(host: String, port: u16) -> Hover {
        Hover {
            host: Ipv4Addr::from_str(&host).expect("IP address expected!"),
            port,
            node: Option::None,
            started: false,
        }
    }

    pub fn start(&mut self) {
        let node = Node::new(self.host.clone(), self.port.clone());
        node.start();
        self.node = Option::from(node);
    }
}

struct Node {
    node_id: String,
    connection_service: ConnectionService,
    multicast_service: MulticastService,
}

impl Node {
    fn new(host: Ipv4Addr, port: u16) -> Node {
        let node_id = String::from("some_string_id(uuid)"); //TODO: generate later

        let connection_service = ConnectionService::new(host, port);

        /**Get multicast configs from config object*/
        let multicast_addr = Ipv4Addr::new(228, 0, 0, 1);
        let multicast_port: u16 = 2403;
        let multicast_service = MulticastService::new(multicast_addr, multicast_port);

        Node {
            node_id,
            connection_service,
            multicast_service,
        }
    }

    fn start(&self) {
         self.connection_service.start();
        self.multicast_service.start();

        println!("Node has been started!");
    }
}