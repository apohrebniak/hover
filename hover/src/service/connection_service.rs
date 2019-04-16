extern crate socket2;

use std::cell::RefCell;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::common::{Address, NodeMeta};
use crate::service::Service;

/**Connection service*/
pub struct ConnectionService {
    local_node_meta: NodeMeta,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl ConnectionService {
    pub fn new(local_node_meta: NodeMeta) -> ConnectionService {
        ConnectionService {
            local_node_meta,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl Service for ConnectionService {
    fn start(&self) {
        let addr = &self.local_node_meta.addr;
        let tcp_listener = TcpListener::bind((addr.ip, addr.port)).expect("Can't create node!");

        //set running state
        let running = self.running.clone();
        running.store(true, Ordering::Relaxed);
        //create a connection thread
        let worker_thread_handle = std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                println!("Connection loop started!");
                let stream = tcp_listener.accept();
                match stream {
                    Ok((stream, addr)) => {
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
