extern crate socket2;

use std::cell::RefCell;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use self::socket2::Socket;
use crate::common::{Address, Message, NodeMeta};
use crate::events::{Event, EventListener, EventLoop};
use crate::serialize;
use crate::service::Service;
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

    fn start_inner(&self) {
        let running = self.running.store(true, Ordering::Relaxed);

        let tcp_listener = self
            .build_inbound_socket(&self.local_node_meta.addr)
            .unwrap();

        let thread_handler = self.listen(tcp_listener).unwrap();

        //set handle to service. Now service is a thread owner
        self.worker_thread_handle
            .lock()
            .unwrap()
            .replace(thread_handler);
        dbg!("Connection service started");
    }

    fn build_inbound_socket(&self, addr: &Address) -> io::Result<TcpListener> {
        TcpListener::bind((addr.ip, addr.port))
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
                                    println!("Read message: {:?}", msg);
                                    let event = Event::MessageIn { msg: Arc::new(msg) };

                                    loop_.write().unwrap().post_event(event);
                                }
                                Err(_) => {
                                    eprintln!("Error while reading message structure");
                                }
                            };
                        }
                        Err(_) => {}
                        _ => {
                            dbg!("Read 0 bytes");
                        }
                    },
                    Err(_) => {
                        eprintln!("Error");
                    }
                }
            }
        });

        Ok(thread_handle)
    }
}

impl Service for ConnectionService {
    fn start(&self) {
        self.start_inner();
    }
}

impl EventListener for ConnectionService {
    fn on_event(&self, event: Event) {
        match event {
            Event::MessageOut { msg } => {
                println!("Damn Boiii");
            }
            _ => {}
        }
    }
}
