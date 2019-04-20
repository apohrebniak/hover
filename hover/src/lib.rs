use std::net::*;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use common::Address;
use message::MessagingService;
use service::cluster_service::MembershipService;
use service::connection_service::ConnectionService;
use service::discovery_service::DiscoveryService;
use service::Service;

use crate::common::{Message, NodeMeta};
use crate::events::EventLoop;
use crate::message::MessageDispatcher;
use core::borrow::{Borrow, BorrowMut};
use std::error::Error;
use uuid::Uuid;

mod cluster;
pub mod common;
pub mod events;
pub mod message;
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

    pub fn get_cluster_service(&self) -> Result<Arc<RwLock<MembershipService>>, &str> {
        match self.node {
            Some(ref node) => Ok(node.membership_service.clone()),
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

    pub fn add_msg_listener<F>(&mut self, f: F) -> Result<&Hover, Box<()>>
    where
        F: Fn(Arc<Message>) -> () + 'static + Send + Sync,
    {
        match self.node {
            Some(ref mut n) => match n.add_msg_listener(f) {
                Ok(_) => Ok(self),
                Err(_) => Err(Box::new(())),
            },
            None => Err(Box::new(())),
        }
    }

    pub fn subscribe_for_topic(&mut self) -> Result<&Hover, Box<()>> {
        match self.node {
            Some(ref mut n) => match n.subscribe_for_topic() {
                Ok(_) => Ok(self),
                Err(_) => Err(Box::new(())),
            },
            None => Err(Box::new(())),
        }
    }
}

/**Representation of the Hover node*/
struct Node {
    meta: NodeMeta,
    connection_service: Arc<RwLock<ConnectionService>>,
    discovery_service: DiscoveryService,
    messaging_service: MessagingService,
    membership_service: Arc<RwLock<MembershipService>>,
    message_dispatcher: Arc<RwLock<MessageDispatcher>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl Node {
    fn new(host: Ipv4Addr, port: u16) -> Node {
        let node_id = Uuid::new_v4();

        let node_meta = NodeMeta {
            id: node_id,
            addr: Address { ip: host, port },
        };

        /**Get multicast configs from config object*/ //TODO: config object
        let multicast_addr = Address {
            ip: Ipv4Addr::new(228, 0, 0, 1),
            port: 2403,
        };

        let event_loop = Arc::new(RwLock::new(EventLoop::new()));

        let membership_service = Arc::new(RwLock::new(MembershipService::new()));

        let connection_service = Arc::new(RwLock::new(ConnectionService::new(
            node_meta.clone(),
            event_loop.clone(),
        )));

        let discovery_service =
            DiscoveryService::new(node_meta.clone(), multicast_addr, event_loop.clone());

        let messaging_service =
            MessagingService::new(node_meta.clone(), membership_service.clone());

        let message_dispatcher = Arc::new(RwLock::new(MessageDispatcher::new()));

        event_loop
            .write()
            .unwrap()
            .add_listener(membership_service.clone())
            .unwrap()
            .add_listener(connection_service.clone())
            .unwrap()
            .add_listener(message_dispatcher.clone())
            .unwrap();

        Node {
            meta: node_meta.clone(),
            connection_service,
            discovery_service,
            messaging_service,
            membership_service,
            message_dispatcher,
            event_loop,
        }
    }

    fn start(&self) {
        self.connection_service.read().unwrap().start();
        self.discovery_service.start();

        self.event_loop.read().unwrap().start();

        println!("Node has been started!");
    }

    fn add_msg_listener<F>(&mut self, f: F) -> Result<(), Box<()>>
    where
        F: Fn(Arc<Message>) -> () + 'static + Send + Sync,
    {
        match self.message_dispatcher.write().unwrap().add_msg_listener(f) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(())),
        }
    }

    fn subscribe_for_topic(&mut self) -> Result<(), Box<()>> {
        match self
            .message_dispatcher
            .write()
            .unwrap()
            .subscribe_for_topic()
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(())),
        }
    }
}
