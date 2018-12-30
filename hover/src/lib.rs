mod service;

use std::net;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::service::{ConnectionService, MulticastService, RunnableService};

pub struct Hover {
    host: String, //TODO: change to InetAddress
    port: u16,
    node: Option<Node>,
    started: bool,
}

impl Hover {
    pub fn new(host: String, port: u16) -> Hover {
        Hover {
            host,
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
    host: String,
    port: u16,
    running: bool, //TODO: move to connection service
    connection_service: ConnectionService,
    multicasts_service: MulticastService,
}

impl Node {
    fn new(host: String, port: u16) -> Node {
        let node_id = String::from("some_string_id(uuid)"); //TODO: generate later

        let tcp_listener = net::TcpListener::bind((host.as_ref(), port)).unwrap(); //TODO: return result
        let connection_service = ConnectionService::new(tcp_listener);

        let multicasts_service = MulticastService { running: false };

        Node {
            node_id,
            host,
            port,
            running: false,
            connection_service,
            multicasts_service,
        }
    }

    fn start(&self) {
        self.connection_service.start();
        self.multicasts_service.start();

        println!("Node has been started!");
    }
}