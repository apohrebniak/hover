use std::net::*;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use common::Address;
use service::cluster_service::ClusterService;
use service::connection_service::ConnectionService;
use service::discovery_service::DiscoveryService;
use service::messaging_service::MessagingService;
use service::Service;

use crate::common::NodeMeta;
use crate::events::EventLoop;

mod cluster;
pub mod common;
pub mod events;
pub mod serialize;
pub mod service;

/**Main API for using service*/
pub struct Hover {
    address: Address,
    node: Option<Node>,
    started: bool,
}

impl Hover {
    pub fn new(host: String, port: u16) -> Hover {
        let addr = Address {
            ip: Ipv4Addr::from_str(&host).expect("IP address expected!"),
            port,
        };

        Hover {
            address: addr,
            node: Option::None,
            started: false,
        }
    }

    pub fn get_cluster_service(&self) -> Result<&ClusterService, &str> {
        match self.node {
            Some(ref node) => Ok(&node.cluster_service),
            None => Err("Node is not initialized!"),
        }
    }

    pub fn get_messaging_service(&self) -> Result<&MessagingService, &str> {
        match self.node {
            Some(ref node) => Ok(&node.messaging_service),
            None => Err("Node is not initialized!"),
        }
    }

    pub fn start(&mut self) -> Result<(), &str> {
        match self.started {
            true => Err("Hover is already started!"),
            false => {
                let node = Node::new(self.address.ip.clone(), self.address.port.clone());
                node.start();
                self.node = Option::from(node);
                self.started = true;
                Ok(())
            }
        }
    }
}

/**Representation of the Hover node*/
struct Node {
    meta: NodeMeta,
    connection_service: ConnectionService,
    discovery_service: DiscoveryService,
    messaging_service: MessagingService,
    cluster_service: Arc<ClusterService>,
    event_loop: Arc<Mutex<EventLoop>>,
}

impl Node {
    fn new(host: Ipv4Addr, port: u16) -> Node {
        let node_id = String::from("some_string_id(uuid)"); //TODO: generate later

        let node_meta = NodeMeta {
            id: node_id,
            addr: Address { ip: host, port },
        };

        let connection_service = ConnectionService::new(node_meta.clone());

        /**Get multicast configs from config object*/ //TODO: config object
        let multicast_addr = Address {
            ip: Ipv4Addr::new(228, 0, 0, 1),
            port: 2403,
        };

        let event_loop = Arc::new(Mutex::new(EventLoop::new()));

        let discovery_service =
            DiscoveryService::new(node_meta.clone(), multicast_addr, event_loop.clone());

        let messaging_service = MessagingService::new();
        let cluster_service = Arc::new(ClusterService::new());

        event_loop
            .lock()
            .unwrap()
            .add_listener(cluster_service.clone());

        Node {
            meta: node_meta.clone(),
            connection_service,
            discovery_service,
            messaging_service,
            cluster_service,
            event_loop,
        }
    }

    fn start(&self) {
        self.connection_service.start();
        self.discovery_service.start();

        self.event_loop.lock().unwrap().start();

        println!("Node has been started!");
    }
}
