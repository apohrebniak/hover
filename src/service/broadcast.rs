extern crate chashmap;
extern crate rand;
extern crate socket2;

use std::io::Read;
use std::net::Ipv4Addr;
use std::net::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

use serde::{Deserialize, Serialize};
use socket2::*;

use self::rand::seq::{IteratorRandom, SliceRandom};
use crate::common::{Address, BroadcastPayload, MessageType, NodeMeta};
use crate::events::Event::{JoinIn, JoinOut, LeftIn};
use crate::events::{Event, EventListener, EventLoop};
use crate::message::MessagingService;
use crate::serialize;
use crate::service::membership::MembershipService;
use crate::service::Service;
use crossbeam_channel::{Receiver, Sender};
use std::collections::btree_set::BTreeSet;
use uuid::Uuid;

const MULTICAST_INPUT_BUFF_SIZE: usize = 256;

/**Listens on multicast messages. Sends messages via multicast*/
pub struct BroadcastService {
    multicast_address: Address,
    sender_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    handler_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    //multithreaded communication
    sender_channel: Sender<DiscoveryMessage>,
    receiver_channel: Receiver<DiscoveryMessage>,
    //gossip
    gossip: Arc<GossipProtocol>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl BroadcastService {
    pub fn new(
        local_node_meta: NodeMeta,
        multicast_address: Address,
        membership_service: Arc<RwLock<MembershipService>>,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> BroadcastService {
        let (s, r): (Sender<DiscoveryMessage>, Receiver<DiscoveryMessage>) =
            crossbeam_channel::unbounded();

        let l = event_loop.clone();

        let gossip = Arc::new(GossipProtocol::new(
            membership_service,
            messaging_service,
            event_loop.clone(),
        ));

        BroadcastService {
            multicast_address,
            sender_thread: Arc::new(Mutex::new(Option::None)),
            handler_thread: Arc::new(Mutex::new(Option::None)),
            sender_channel: s,
            receiver_channel: r,
            gossip,
            event_loop,
        }
    }

    fn start_inner(&self) -> Result<(), &str> {
        let multi_addr = self.multicast_address.ip;
        let multi_port = self.multicast_address.port;

        let multi_sock_addr = SockAddr::from(SocketAddrV4::new(multi_addr, multi_port));

        let socket_send = self.build_socket_send(&multi_sock_addr)?;
        let socket_receive = self.build_socket_receive(&multi_addr, multi_port)?;

        let sender_thread = self.start_sending(socket_send)?;
        let handler_thread = self.start_listening(socket_receive)?;

        //set thread handler to service. Service is the thread owner
        self.sender_thread.lock().unwrap().replace(sender_thread);
        self.handler_thread.lock().unwrap().replace(handler_thread);
        println!("[BroadcastService]: Started");

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

    fn start_sending(&self, socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let receiver_channel_ = self.receiver_channel.clone();

        let thread = std::thread::spawn(move || {
            println!("[BroadcastService]: Started sending multicast messages");
            for msg in receiver_channel_.iter() {
                let msg_bytes = serialize::to_bytes(&msg).unwrap();

                match socket.send(msg_bytes.as_slice()) {
                    Ok(_) => {
                        println!("[BroadcastService]: Sent message to multicast group: OK");
                    }
                    Err(_) => eprintln!("[BroadcastService]: Sent message to multicast group: ERR"),
                };
            }
        });

        Ok(thread)
    }

    fn start_listening(&self, socket: Socket) -> Result<std::thread::JoinHandle<()>, &str> {
        let e_loop_ = self.event_loop.clone();

        let thread = std::thread::spawn(move || loop {
            let mut buff = [0u8; MULTICAST_INPUT_BUFF_SIZE];

            match socket.recv_from(&mut buff) {
                Ok((size, ref sockaddr)) if size > 0 => match serialize::from_bytes(&buff) {
                    Ok(msg) => {
                        let event = self::BroadcastService::build_discovery_event(&msg, &sockaddr);
                        e_loop_.read().unwrap().post_event(event);
                    }
                    Err(_) => {}
                },
                Err(_) => eprintln!("[BroadcastService]: Read message via multicast: ERR"),
                _ => {}
            }
        });

        Ok(thread)
    }

    fn build_discovery_event(msg: &DiscoveryMessage, sockaddr: &SockAddr) -> Event {
        let ip = sockaddr.as_inet().map(|i| i.ip().clone()).unwrap();
        let port = sockaddr.as_inet().map(|i| i.port()).unwrap();

        match msg.r#type {
            DiscoveryMessageType::Joined => JoinIn {
                node_meta: msg.node_meta.clone(),
            },
            DiscoveryMessageType::Left => LeftIn {
                node_meta: msg.node_meta.clone(),
            },
        }
    }

    fn send_join_message(&self, node: NodeMeta) {
        let msg = DiscoveryMessage {
            r#type: DiscoveryMessageType::Joined,
            node_meta: node,
        };

        self.sender_channel.send(msg);
    }

    fn send_leave_message(&self, node: NodeMeta) {
        let msg = DiscoveryMessage {
            r#type: DiscoveryMessageType::Left,
            node_meta: node,
        };

        self.sender_channel.send(msg);
    }
}

impl Service for BroadcastService {
    fn start(&self) {
        self.start_inner();
    }
}

impl EventListener for BroadcastService {
    fn on_event(&self, event: Event) {
        match event {
            Event::JoinOut { node_meta } => self.send_join_message(node_meta),
            Event::LeftOut { node_meta } => self.send_leave_message(node_meta),
            Event::BroadcastIn { payload } => self.gossip.handle_received_broadcast(payload),
            Event::BroadcastOut { payload } => self.gossip.send_new_broadcast(payload),
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
enum DiscoveryMessageType {
    // node joined the cluster and ready to pickup connections
    Joined = 0,
    Left = 1, // node is leaving the cluster
}

/**Message that multicasts*/
#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
struct DiscoveryMessage {
    r#type: DiscoveryMessageType,
    node_meta: NodeMeta,
}

/**Gossip protocol implementation and process*/
struct GossipProtocol {
    buffer: chashmap::CHashMap<Uuid, BufferedBroadcast>,
    buffer_keys: RwLock<Vec<Uuid>>,
    membership_service: Arc<RwLock<MembershipService>>,
    messaging_service: Arc<RwLock<MessagingService>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl GossipProtocol {
    fn new(
        membership_service: Arc<RwLock<MembershipService>>,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> GossipProtocol {
        GossipProtocol {
            buffer: chashmap::CHashMap::new(),
            buffer_keys: RwLock::new(Vec::new()),
            membership_service,
            messaging_service,
            event_loop,
        }
    }

    fn start(&self) {
        let mut rng = &mut rand::thread_rng();

        loop {
            let buffered_broadcast = self
                .buffer_keys
                .read()
                .unwrap()
                .choose(&mut rng)
                .and_then(|key| self.buffer.get_mut(key))
                .map(|mut f| {
                    f.ttl -= 1;
                    f
                })
                .unwrap();

            let bytes_to_broadcast = serialize::to_bytes(&buffered_broadcast.payload).unwrap();

            let peers: Vec<NodeMeta> = self
                .membership_service
                .read()
                .map(|ms| ms.get_members())
                .unwrap()
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect(); //TODO: aaaaaaaaa

            for peer in peers.iter() {
                self.messaging_service
                    .read()
                    .unwrap()
                    .send_to_member_type(bytes_to_broadcast.clone(), peer, MessageType::Broadcast)
                    .unwrap();
            }

            self.buffer.retain(|_, value| value.ttl > -10); //TODO
            self.buffer_keys
                .write()
                .unwrap()
                .retain(|key| self.buffer.contains_key(key));

            std::thread::sleep_ms(1000);
        }
    }

    //put message into buffer
    fn send_new_broadcast(&self, payload: Vec<u8>) {
        let broadcast_payload = BroadcastPayload {
            id: uuid::Uuid::new_v4(),
            payload,
        };

        self.add_to_message_buffer(broadcast_payload);
    }

    fn handle_received_broadcast(&self, payload: BroadcastPayload) {
        if !self.buffer_contains(&payload) {
            //TODO: notify handlers
            self.add_to_message_buffer(payload);
        }
    }

    fn add_to_message_buffer(&self, payload: BroadcastPayload) {
        let key = payload.id.clone();

        let buffered_message = BufferedBroadcast {
            ttl: get_fanout(),
            payload,
        };

        self.buffer.insert_new(key.clone(), buffered_message);
        if self.buffer.get(&key).is_some() {
            self.buffer_keys.write().unwrap().push(key)
        }
    }

    fn buffer_contains(&self, payload: &BroadcastPayload) -> bool {
        self.buffer.contains_key(&payload.id)
    }
}

fn get_fanout() -> i32 {
    2 //TODO
}

struct BufferedBroadcast {
    ttl: i32,
    payload: BroadcastPayload,
}
