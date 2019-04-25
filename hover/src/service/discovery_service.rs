extern crate socket2;

use std::io::Read;
use std::net::Ipv4Addr;
use std::net::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

use serde::{Deserialize, Serialize};
use socket2::*;

use crate::common::{Address, NodeMeta};
use crate::events::Event::DiscoveryIn;
use crate::events::{Event, EventLoop};
use crate::serialize;
use crate::service::Service;

const MULTICAST_INPUT_BUFF_SIZE: usize = 256;

/**Discovery service*/
pub struct DiscoveryService {
    local_node_meta: NodeMeta,
    multicast_address: Address,
    running: Arc<AtomicBool>,
    sender_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    handler_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl DiscoveryService {
    pub fn new(
        local_node_meta: NodeMeta,
        multicast_address: Address,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> DiscoveryService {
        DiscoveryService {
            local_node_meta,
            multicast_address,
            running: Arc::new(AtomicBool::default()),
            sender_thread: Arc::new(Mutex::new(Option::None)),
            handler_thread: Arc::new(Mutex::new(Option::None)),
            event_loop,
        }
    }

    fn start_inner(&self) -> Result<(), &str> {
        let running = self.running.store(true, Ordering::Relaxed);

        let multi_addr = self.multicast_address.ip;
        let multi_port = self.multicast_address.port;

        let multi_sock_addr = SockAddr::from(SocketAddrV4::new(multi_addr, multi_port));

        let socket_send = self.build_socket_send(&multi_sock_addr)?;
        let socket_receive = self.build_socket_receive(&multi_addr, multi_port)?;

        let sender_thread = self.join(socket_send)?;
        let handler_thread = self.multicast_handler(socket_receive)?;

        //        set thread handler to service. Service is the thread owner
        self.sender_thread.lock().unwrap().replace(sender_thread);
        self.handler_thread.lock().unwrap().replace(handler_thread);
        println!("[DiscoveryService]: Discovery service started");

        Ok(())
    }

    fn build_socket_send(&self, multi_sock_addr: &SockAddr) -> Result<Socket, &str> {
        let socket =
            socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket.connect(multi_sock_addr);

        Ok(socket)
    }

    fn build_socket_receive(&self, multi_addr: &Ipv4Addr, multi_port: u16) -> Result<Socket, &str> {
        let socket =
            socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket.set_reuse_port(true);
        socket
            .bind(&SockAddr::from(SocketAddrV4::new(
                Ipv4Addr::UNSPECIFIED,
                multi_port,
            )))
            .unwrap();
        socket.join_multicast_v4(multi_addr, &Ipv4Addr::UNSPECIFIED);

        Ok(socket)
    }

    //TODO: broadcast leaved nodes
    fn join(&self, socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let running = self.running.clone();

        let msg = DiscoveryMessage {
            r#type: DiscoveryMessageType::Join,
            node_meta: self.local_node_meta.clone(),
        };

        let msg = serialize::to_bytes(&msg).unwrap();

        let thread = std::thread::spawn(move || {
            println!("[DiscoveryService]: Started sending multicast messages");
            while running.load(Ordering::Relaxed) {
                match socket.send(msg.as_slice()) {
                    Ok(_) => {
                        println!("[DiscoveryService]: Sent message to multicast group: OK");
                    }
                    Err(_) => eprintln!("[DiscoveryService]: Sent message to multicast group: ERR"),
                };
                std::thread::sleep_ms(5000); //TODO: change to interval setting
            }
        });

        Ok(thread)
    }

    fn multicast_handler(&self, socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let running_ = self.running.clone();
        let e_loop_ = self.event_loop.clone();

        let thread = std::thread::spawn(move || {
            while running_.load(Ordering::Relaxed) {
                let mut buff = [0u8; MULTICAST_INPUT_BUFF_SIZE];

                match socket.recv_from(&mut buff) {
                    Ok((size, ref sockaddr)) if size > 0 => match serialize::from_bytes(&buff) {
                        Ok(msg) => {
                            let event =
                                self::DiscoveryService::build_discovery_event(&msg, &sockaddr);
                            e_loop_.write().unwrap().post_event(event);
                        }
                        Err(_) => {}
                    },
                    Err(_) => eprintln!("[DiscoveryService]: Read message via multicast: ERR"),
                    _ => {}
                }
            }
        });

        Ok(thread)
    }

    //TODO: leaving
    fn build_discovery_event(msg: &DiscoveryMessage, sockaddr: &SockAddr) -> Event {
        let ip = sockaddr.as_inet().map(|i| i.ip().clone()).unwrap();
        let port = sockaddr.as_inet().map(|i| i.port()).unwrap();

        DiscoveryIn {
            node_meta: msg.node_meta.clone(),
        }
    }
}

impl Service for DiscoveryService {
    fn start(&self) {
        self.start_inner();
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
enum DiscoveryMessageType {
    // node joined the cluster and ready to pickup connections
    Join = 0,
    Leave = 1, // node is leaving the cluster
}

/**Message that multicasts*/
#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
struct DiscoveryMessage {
    r#type: DiscoveryMessageType,
    node_meta: NodeMeta,
}
