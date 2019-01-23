extern crate socket2;

use std::cell::RefCell;
use std::collections::HashSet;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use socket2::*;

use crate::cluster::Member;
use crate::common::{Address, Message};

/**Common trait for all runnable services*/
pub trait Service {
    fn start(&self);
}

/**Connection service*/
pub struct ConnectionService {
    address: Address,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl ConnectionService {
    pub fn new(host: Ipv4Addr, port: u16) -> ConnectionService {
        let addr = Address { ip: host, port };
        ConnectionService {
            address: addr,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl Service for ConnectionService {
    fn start(&self) {
        let tcp_listener =
            TcpListener::bind((self.address.ip, self.address.port)).expect("Can't create node!");

        //set running state
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);
        //create a connection thread
        let worker_thread_handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                println!("Connection loop started!");
                let stream = tcp_listener.accept();
                match stream {
                    Ok(s) => {
                        println!("Connection established!");
                    }
                    _ => println!("Error"),
                }
            }
        });

        //set handle to service. Now service is a thread owner
        self.worker_thread_handle
            .borrow_mut()
            .replace(worker_thread_handle);
        println!("Connection service started");
    }
}

/**Discovery service*/
pub struct DiscoveryService {
    multicast_address: Address,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl DiscoveryService {
    pub fn new(multicast_addr: Ipv4Addr, multicast_port: u16) -> DiscoveryService {
        let addr = Address {
            ip: multicast_addr,
            port: multicast_port,
        };
        DiscoveryService {
            multicast_address: addr,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl Service for DiscoveryService {
    fn start(&self) {
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);

        let addr = SockAddr::from(SocketAddrV4::new(
            self.multicast_address.ip,
            self.multicast_address.port,
        ));

        let worker_thread_handler = std::thread::spawn(move || {
            let multicast_socket =
                socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();

            multicast_socket.connect(&addr);

            let msg = "Hello multicast!";
            while running.load(Ordering::Relaxed) {
                match multicast_socket.send(msg.as_bytes()) {
                    Ok(_) => println!("OK"),
                    Err(_) => eprintln!("err"),
                };
                std::thread::sleep_ms(1000);
            }
        });

        //create a connection thread
        let worker_thread_handler = std::thread::spawn(move || {});

        //set thread handler to service. Service is the thread owner
        self.worker_thread_handle
            .borrow_mut()
            .replace(worker_thread_handler);
        println!("Multicast service started")
    }
}

/**Service for sending messages across cluster.*/
pub struct MessagingService {}

impl MessagingService {
    pub fn new() -> MessagingService {
        MessagingService {}
    }

    pub fn send_to_address(&self, msg: Message, address: Address) -> Result<(), &str> {
        dbg!("Sent message to address");
        Ok(())
    }

    pub fn send_to_member(&self, msg: Message, member: Member) -> Result<(), &str> {
        dbg!("Send message to member");
        Ok(())
    }

    pub fn send_to_address_receive(&self, msg: Message, address: Address) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {})
    }

    pub fn send_to_member_receive(&self, msg: Message, member: Member) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {})
    }

    pub fn broadcast(&self, msg: Message) -> Result<(), &str> {
        dbg!("Broadcasted");
        Ok(())
    }

    pub fn multicast_to_addresses(
        &self,
        msg: Message,
        addresses: HashSet<Address>,
    ) -> Result<(), &str> {
        dbg!("multicast to addresses");
        Ok(())
    }

    pub fn multicast_to_members(&self, msg: Message, members: HashSet<Member>) -> Result<(), &str> {
        dbg!("multicast to members");
        Ok(())
    }
}

impl Service for MessagingService {
    fn start(&self) {
        println!("Messaging service started")
    }
}

pub struct ClusterService {}

impl ClusterService {
    pub fn new() -> ClusterService {
        ClusterService {}
    }

    pub fn get_members() -> HashSet<Member> {
        HashSet::new() //TODO
    }

    pub fn get_member_by_id(member_id: &str) -> Option<Member> {
        None
    }

    pub fn get_member_by_address(address: Address) -> Option<Member> {
        None
    }
}

impl Service for ClusterService {
    fn start(&self) {
        println!("Cluster service started")
    }
}
