use std::net::*;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use broadcast::BroadcastService;
use common::Address;
use connection::ConnectionService;
use membership::MembershipService;
use message::MessagingService;

use crate::common::{BroadcastMessage, Message, NodeMeta};
use crate::config::HoverConfig;
use crate::discovery::DiscoveryProvider;
use crate::events::{EventListener, EventLoop};
use crate::message::MessageDispatcher;
use core::borrow::{Borrow, BorrowMut};
use std::error::Error;
use uuid::Uuid;

pub mod broadcast;
pub mod common;
pub mod config;
pub mod connection;
pub mod discovery;
pub mod events;
pub mod membership;
pub mod message;
pub mod serialize;

/**Main API for using service*/
pub struct Hover {
    node: Option<Node>,
    config: HoverConfig,
    started: bool,
}

impl Hover {
    fn new(conf: config::HoverConfig) -> Result<Hover, Box<Error>> {
        println!("Initializing with config: {:?}", conf);

        let hover = Hover {
            node: Option::None,
            config: conf,
            started: false,
        };

        Ok(hover)
    }

    pub fn default() -> Result<Hover, Box<Error>> {
        let conf = config::HoverConfig::default()?;
        self::Hover::new(conf)
    }

    pub fn with_conf(conf: config::HoverConfig) -> Result<Hover, Box<Error>> {
        self::Hover::new(conf)
    }

    pub fn with_conf_path(path: &str) -> Result<Hover, Box<Error>> {
        let conf = config::HoverConfig::from_file(path)?;
        self::Hover::new(conf)
    }

    pub fn get_node_id(&self) -> Option<Uuid> {
        match self.node {
            Some(ref node) => Some(node.meta.id.clone()),
            None => None,
        }
    }

    pub fn get_cluster_service(&self) -> Result<Arc<RwLock<MembershipService>>, &str> {
        match self.node {
            Some(ref node) => Ok(node.membership_service.clone()),
            None => Err("Node is not initialized!"),
        }
    }

    pub fn get_messaging_service(&self) -> Result<Arc<RwLock<MessagingService>>, &str> {
        match self.node {
            Some(ref node) => Ok(node.messaging_service.clone()),
            None => Err("Node is not initialized!"),
        }
    }

    pub fn start(&mut self) -> Result<(), &str> {
        match self.started {
            true => Err("Hover is already started!"),
            false => {
                let node = Node::new(self.config.clone());
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

    pub fn add_broadcast_listener<F>(&self, f: F) -> Result<&Hover, Box<()>>
    where
        F: Fn(Arc<BroadcastMessage>) -> () + 'static + Send + Sync,
    {
        match self.node {
            Some(ref n) => match n.add_broadcast_listener(f) {
                Ok(_) => Ok(self),
                Err(_) => Err(Box::new(())),
            },
            None => Err(Box::new(())),
        }
    }

    pub fn add_event_listener<T>(&self, listener: T) -> Result<&Hover, Box<()>>
    where
        T: EventListener + Send + Sync + 'static,
    {
        match self.node {
            Some(ref node) => match node.add_event_listener(listener) {
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
    config: HoverConfig,
    connection_service: Arc<RwLock<ConnectionService>>,
    broadcast_service: Arc<RwLock<BroadcastService>>,
    messaging_service: Arc<RwLock<MessagingService>>,
    membership_service: Arc<RwLock<MembershipService>>,
    message_dispatcher: Arc<RwLock<MessageDispatcher>>,
    discovery_provider: Arc<RwLock<DiscoveryProvider>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl Node {
    fn new(conf: HoverConfig) -> Node {
        let node_id = Uuid::new_v4();

        let node_meta = NodeMeta {
            id: node_id,
            addr: Address {
                ip: Ipv4Addr::from_str(conf.address.as_str()).unwrap(),
                port: conf.port,
            },
        };

        /**Get multicast configs from config object*/
        let multicast_addr = Address {
            ip: Ipv4Addr::from_str(conf.discovery.multicast_group.as_str()).unwrap(),
            port: conf.discovery.multicast_port,
        };

        let event_loop = Arc::new(RwLock::new(EventLoop::new()));

        let connection_service = Arc::new(RwLock::new(ConnectionService::new(
            node_meta.clone(),
            event_loop.clone(),
        )));

        let message_dispatcher = Arc::new(RwLock::new(MessageDispatcher::new(event_loop.clone())));

        let messaging_service = Arc::new(RwLock::new(MessagingService::new(
            node_meta.clone(),
            message_dispatcher.clone(),
            event_loop.clone(),
        )));

        let membership_service = Arc::new(RwLock::new(MembershipService::new(
            node_meta.clone(),
            conf.discovery.clone(),
            messaging_service.clone(),
            event_loop.clone(),
        )));

        let discovery_provider = Arc::new(RwLock::new(DiscoveryProvider::new(
            node_meta.clone(),
            conf.discovery.clone(),
            membership_service.clone(),
            event_loop.clone(),
        )));

        let broadcast_service = Arc::new(RwLock::new(BroadcastService::new(
            node_meta.clone(),
            conf.broadcast.clone(),
            multicast_addr,
            membership_service.clone(),
            messaging_service.clone(),
            event_loop.clone(),
        )));

        event_loop
            .write()
            .unwrap()
            .add_listener(membership_service.clone())
            .unwrap()
            .add_listener(message_dispatcher.clone())
            .unwrap()
            .add_listener(broadcast_service.clone())
            .unwrap()
            .add_listener(discovery_provider.clone())
            .unwrap();

        Node {
            meta: node_meta.clone(),
            config: conf,
            connection_service,
            broadcast_service,
            messaging_service,
            membership_service,
            message_dispatcher,
            discovery_provider,
            event_loop,
        }
    }

    fn start(&self) {
        self.event_loop.read().unwrap().start();

        self.connection_service.read().unwrap().start();
        self.broadcast_service.read().unwrap().start();
        self.discovery_provider.read().unwrap().start();
        self.membership_service.read().unwrap().start();

        println!("[Node]: Started");
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

    fn add_broadcast_listener<F>(&self, f: F) -> Result<(), Box<()>>
    where
        F: Fn(Arc<BroadcastMessage>) -> () + 'static + Send + Sync,
    {
        match self
            .broadcast_service
            .write()
            .unwrap()
            .add_broadcast_listener(f)
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(())),
        }
    }

    fn add_event_listener<T>(&self, listener: T) -> Result<(), Box<()>>
    where
        T: EventListener + Send + Sync + 'static,
    {
        let lis = Arc::new(RwLock::new(listener));
        match self.event_loop.read() {
            Ok(l) => l.add_listener(lis).map(|_| ()).map_err(|_| Box::new(())),
            Err(_) => Err(Box::new(())),
        }
    }
}
