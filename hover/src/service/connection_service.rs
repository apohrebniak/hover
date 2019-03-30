extern crate socket2;

use socket2::*;

use std::cell::RefCell;
use std::collections::HashSet;
use std::error::Error;
use std::io::Read;
use std::net::*;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::cluster::Member;
use crate::common::{Address, Message};
use crate::service::Service;

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