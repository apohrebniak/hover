extern crate socket2;

use std::cell::RefCell;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener, TcpStream, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use socket2::*;

/**Common trait for all runnable services*/
pub trait RunnableService {
    fn start(&self);
}

/**Connection service*/
pub struct ConnectionService {
    host: Ipv4Addr,
    port: u16,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl ConnectionService {
    pub fn new(host: Ipv4Addr, port: u16) -> ConnectionService {
        ConnectionService {
            host,
            port,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl RunnableService for ConnectionService {
    fn start(&self) {
        let tcp_listener = TcpListener::bind((self.host, self.port)).expect("Can't create node!");

        //set running state
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);

        //create a connection thread
        let worker_thread_handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                println!("Connection loop started!");
                let stream = tcp_listener.accept();
                match stream {
                    Ok(stream) => {
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

/**Multicast service*/
pub struct MulticastService {
    multicast_group: Ipv4Addr,
    multicast_port: u16,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl MulticastService {
    pub fn new(multicast_addr: Ipv4Addr, multicast_port: u16) -> MulticastService {
        MulticastService {
            multicast_group: multicast_addr,
            multicast_port,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl RunnableService for MulticastService {
    fn start(&self) {
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);

        let addr = SockAddr::from(SocketAddrV4::new(self.multicast_group, self.multicast_port));

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
