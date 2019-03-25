use std::net::*;
use std::str::FromStr;

use common::Address;
use service::{ClusterService, ConnectionService, DiscoveryService, MessagingService, Service};

mod cluster;
pub mod common;
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
    node_id: String,
    connection_service: ConnectionService,
    discovery_service: DiscoveryService,
    messaging_service: MessagingService,
    cluster_service: ClusterService,
}

impl Node {
    fn new(host: Ipv4Addr, port: u16) -> Node {
        let node_id = String::from("some_string_id(uuid)"); //TODO: generate later

        let connection_service = ConnectionService::new(host, port);

        /**Get multicast configs from config object*/ //TODO: config object
        let multicast_addr = Ipv4Addr::new(228, 0, 0, 1);
        let multicast_port: u16 = 2403;
        let discovery_service = DiscoveryService::new(multicast_addr, multicast_port);

        let messaging_service = MessagingService::new();
        let cluster_service = ClusterService::new();

        Node {
            node_id,
            connection_service,
            discovery_service,
            messaging_service,
            cluster_service,
        }
    }

    fn start(&self) {
        self.connection_service.start();
        self.discovery_service.start();

        println!("Node has been started!");
    }
}
