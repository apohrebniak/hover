extern crate socket2;

use socket2::*;

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::io::Read;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::cluster::Member;
use crate::common::{Address, Message};
use crate::service::Service;

/**Connection service*/
pub struct ConnectionService {
    local_address: Address,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<RefCell<Option<std::thread::JoinHandle<()>>>>,
}

impl ConnectionService {
    pub fn new(local_address: Address) -> ConnectionService {
        ConnectionService {
            local_address,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(RefCell::new(Option::None)),
        }
    }
}

impl Service for ConnectionService {
    fn start(&self) {
        let tcp_listener = TcpListener::bind((self.local_address.ip, self.local_address.port))
            .expect("Can't create node!");

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
