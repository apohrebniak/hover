use std::cell::RefCell;
use std::net;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ConnectionService {
    running: Arc<AtomicBool>,
    tcp_listener: Arc<net::TcpListener>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl ConnectionService {
    pub fn new(tcp_listener: net::TcpListener) -> ConnectionService {
        ConnectionService {
            running: Arc::new(AtomicBool::default()),
            tcp_listener: Arc::new(tcp_listener),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }

    fn start_internal(&self) {
        //set running state
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);

        let tcp_listener = self.tcp_listener.clone();

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

        //set handle to node. Now node is a thread owner
        self.worker_thread_handle
            .borrow_mut()
            .replace(worker_thread_handle);
    }
}

impl RunnableService for ConnectionService {
    fn start(&self) {
        println!("Connection service start");
        self.start_internal();
    }
}

pub struct MulticastService {
    pub running: bool,
}

impl RunnableService for MulticastService {
    fn start(&self) {
        println!("Multicast service start")
    }
}

pub trait RunnableService {
    fn start(&self);
}
