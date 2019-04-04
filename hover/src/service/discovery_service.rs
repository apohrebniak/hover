extern crate socket2;
extern crate serde_json;
extern crate serde_repr;

use socket2::*;

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::io::Read;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde_repr::{Serialize_repr, Deserialize_repr};
use serde::{Deserialize, Serialize};

use crate::cluster::Member;
use crate::common::{Address, Message};
use crate::service::Service;

/**Discovery service*/
pub struct DiscoveryService {
    multicast_address: Address,
    running: Arc<AtomicBool>,
    sender_thread: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
    listener_thread: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl DiscoveryService {
    pub fn new(multicast_address: Address) -> DiscoveryService {
        DiscoveryService {
            multicast_address,
            running: Arc::new(AtomicBool::default()),
            sender_thread: Arc::new(RefCell::new(Option::None)),
            listener_thread: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl Service for DiscoveryService {
    fn start(&self) { //TODO: refactor this
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);

        let multi_addr = self.multicast_address.ip;
        let multi_port = self.multicast_address.port;

        let multi_sock_addr = SockAddr::from(SocketAddrV4::new(
            multi_addr,
            multi_port,
        ));

        let socket_send = socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket_send.connect(&multi_sock_addr);

        let mut socket_receive = socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket_receive.set_reuse_port(true);
        socket_receive.bind(&SockAddr::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, multi_port))).unwrap();
        socket_receive.join_multicast_v4(&multi_addr, &Ipv4Addr::UNSPECIFIED);

        let sender_thread = std::thread::spawn(move || {
            let msg = "Hello multicast!"; //TODO
            dbg!("Started sending multicast messages");
            while running.load(Ordering::Relaxed) {
                match socket_send.send(msg.as_bytes()) { //TODO: filter destination
                    Ok(_) => { dbg!("Sent message to multicast group: OK"); }
                    Err(_) => eprintln!("Sent message to multicast group: ERR"),
                };
                std::thread::sleep_ms(1000); //TODO: change to interval setting
            }
        });

        let running = self.running.clone();

        let listener_thread = std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match socket_receive.read(&mut buf) {
                    Ok(size) => {
                        println!("Received {} bytes via multicast", size);
                    }
                    Err(_) => eprintln!("Read message via multicast: ERR"),
                }
            }
        });

//        set thread handler to service. Service is the thread owner
        self.sender_thread
            .borrow_mut()
            .replace(sender_thread);
        self.listener_thread
            .borrow_mut()
            .replace(listener_thread);
        dbg!("Multicast service started");
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Hash)]
#[repr(u8)]
enum DiscoveryMessageType {
    CONNECTION_TRY = 0,
    CONNECTION_OK = 1,
    CONNECTION_DUPLICATE = 2,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
struct DiscoveryMessage {
    r#type: DiscoveryMessageType,
    node_id: String,
}