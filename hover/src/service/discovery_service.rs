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
    handler_thread: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl DiscoveryService {
    pub fn new(multicast_address: Address) -> DiscoveryService {
        DiscoveryService {
            multicast_address,
            running: Arc::new(AtomicBool::default()),
            sender_thread: Arc::new(RefCell::new(Option::None)),
            handler_thread: Arc::new(RefCell::new(Option::None)),
        }
    }

    fn start_inner(&self) -> Result<(), &str> {
        let running = self.running.store(true, Ordering::Relaxed);

        let multi_addr = self.multicast_address.ip;
        let multi_port = self.multicast_address.port;

        let multi_sock_addr = SockAddr::from(SocketAddrV4::new(
            multi_addr,
            multi_port,
        ));

        let socket_send = self.build_socket_send(&multi_sock_addr)?;
        let mut socket_receive = self.build_socket_receive(&multi_addr, multi_port)?;

        let sender_thread = self.join(socket_send)?;
        let handler_thread = self.multicast_handler(socket_receive)?;

//        set thread handler to service. Service is the thread owner
        self.sender_thread
            .borrow_mut()
            .replace(sender_thread);
        self.handler_thread
            .borrow_mut()
            .replace(handler_thread);
        dbg!("Multicast service started");

        Ok(())
    }

    fn build_socket_send(&self, multi_sock_addr: &SockAddr) -> Result<Socket, &str> {
        let mut socket = socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket.connect(multi_sock_addr);

        Ok(socket)
    }

    fn build_socket_receive(&self, multi_addr: &Ipv4Addr, multi_port: u16) -> Result<Socket, &str> {
        let mut socket = socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket.set_reuse_port(true);
        socket.bind(&SockAddr::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, multi_port))).unwrap();
        socket.join_multicast_v4(multi_addr, &Ipv4Addr::UNSPECIFIED);

        Ok(socket)
    }

    fn join(&self, socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let running = self.running.clone();

        let msg = DiscoveryMessage{
            r#type: DiscoveryMessageType::Join,
            node_id: String::from("123-456-789"),
        };

        let msg = serde_json::to_string(&msg)
            .map(|json| json.into_bytes())
            .unwrap();

        let thread = std::thread::spawn(move || {
            dbg!("Started sending multicast messages");
            while running.load(Ordering::Relaxed) {
                match socket.send(msg.as_slice()) { //TODO: filter destination
                    Ok(_) => { dbg!("Sent message to multicast group: OK"); }
                    Err(_) => eprintln!("Sent message to multicast group: ERR"),
                };
                std::thread::sleep_ms(2000); //TODO: change to interval setting
            }
        });

        Ok(thread)
    }

    fn multicast_handler(&self, mut socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let running = self.running.clone();
        //TODO: investigate how to read messages from socket. Should I know the size of it? Should i use the buffer size with the capacity equals to the maximum message size?
        let thread = std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while running.load(Ordering::Relaxed) {
                match socket.read(&mut buf) {
                    Ok(size) => {
                        println!("Received {} bytes via multicast", size);
                    }
                    Err(_) => eprintln!("Read message via multicast: ERR"),
                }
            }
        });

        Ok(thread)
    }
}

impl Service for DiscoveryService {
    fn start(&self) {
        self.start_inner();
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Hash)]
#[repr(u8)]
enum DiscoveryMessageType {
    Join = 0, // node joined the cluster and ready to pickup connections
    Leave = 1, // node is leaving the cluster
}

/**Message that multicasts*/
#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
struct DiscoveryMessage {
    r#type: DiscoveryMessageType,
    node_id: String,
}