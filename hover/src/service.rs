extern crate socket2;

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::io::Read;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
                        dbg!("Connection established!");
                    }
                    _ => {
                        eprintln!("Error");
                    }
                }
            }
        });

        //set handle to service. Now service is a thread owner
        self.worker_thread_handle
            .borrow_mut()
            .replace(worker_thread_handle);
        dbg!("Connection service started");
    }
}

/**Discovery service*/
pub struct DiscoveryService {
    multicast_address: Address,
    running: Arc<AtomicBool>,
    sender_thread: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
    listener_thread: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
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

        let multi_sock = SockAddr::from(SocketAddrV4::new(
            multi_addr,
            multi_port,
        ));

        let socket_send = socket2::Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
        socket_send.connect(&multi_sock);

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

    //TODO: consider subscription topics
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

/**Service that allows to retrieve info about cluster members*/
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
        dbg!("Cluster service started");
    }
}
