extern crate socket2;

use std::cell::RefCell;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use self::socket2::Socket;
use crate::common::{Address, Message, MessageType, NodeMeta};
use crate::events::{Event, EventListener, EventLoop};
use crate::serialize;

use std::error::Error;
use std::io;
use std::io::Read;
use std::thread::JoinHandle;

/**Connection service*/
pub struct ConnectionService {
    local_node_meta: NodeMeta,
    running: Arc<AtomicBool>,
    worker_thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl ConnectionService {
    pub fn new(local_node_meta: NodeMeta, event_loop: Arc<RwLock<EventLoop>>) -> ConnectionService {
        ConnectionService {
            local_node_meta,
            running: Arc::new(AtomicBool::default()),
            worker_thread_handle: Arc::new(Mutex::new(Option::None)),
            event_loop,
        }
    }

    pub fn start(&self) {
        let running = self.running.store(true, Ordering::Relaxed);

        let tcp_listener = self
            .build_inbound_socket(self.local_node_meta.addr.port)
            .unwrap();

        let thread_handler = self.listen(tcp_listener).unwrap();

        //set handle to service. Now service is a thread owner
        self.worker_thread_handle
            .lock()
            .unwrap()
            .replace(thread_handler);
        println!("[ConnectionService]: Started");
    }

    fn build_inbound_socket(&self, port: u16) -> io::Result<TcpListener> {
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
    }

    fn listen(&self, tcp_listener: TcpListener) -> Result<JoinHandle<()>, Box<Error>> {
        let running_ = self.running.clone();
        let loop_ = self.event_loop.clone();

        //create a connection thread
        let thread_handle = std::thread::spawn(move || {
            while running_.load(Ordering::Relaxed) {
                let stream = tcp_listener.accept();
                let mut buff: Vec<u8> = Vec::new();

                match stream {
                    Ok((mut stream, addr)) => match stream.read_to_end(&mut buff) {
                        Ok(size) if size > 0 => {
                            match serialize::from_bytes(buff.as_mut_slice()) {
                                Ok(msg) => {
                                    println!(
                                        "[ConnectionService]: Has read the message: {:?}",
                                        msg
                                    );
                                    let event = Event::MessageIn { msg: Arc::new(msg) };

                                    loop_.read().unwrap().post_event(event);
                                }
                                Err(_) => {
                                    eprintln!("[ConnectionService]: Error while reading message structure");
                                }
                            };
                        }
                        Err(_) => {}
                        _ => {
                            println!("[ConnectionService]: Read 0 bytes");
                        }
                    },
                    Err(_) => {
                        eprintln!("[ConnectionService]: Failed to start listener");
                    }
                }
            }
        });

        Ok(thread_handle)
    }
}
