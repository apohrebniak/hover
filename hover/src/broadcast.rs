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

use self::chashmap::{ReadGuard, WriteGuard};
use self::rand::prelude::ThreadRng;
use self::rand::seq::{IteratorRandom, SliceRandom};
use crate::common::{Address, BroadcastMessage, MessageType, NodeMeta};
use crate::events::Event::{JoinIn, JoinOut, LeftIn};
use crate::events::{Event, EventListener, EventLoop};
use crate::membership::MembershipService;
use crate::message::MessagingService;
use crate::serialize;

use crate::config::{BroadcastConfig, DiscoveryConfig};
use core::borrow::BorrowMut;
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::collections::btree_set::BTreeSet;
use std::time::Duration;
use uuid::Uuid;

const MULTICAST_INPUT_BUFF_SIZE: usize = 256;

/**Listens on multicast messages. Sends messages via multicast*/
pub struct BroadcastService {
    multicast_address: Address,
    sender_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    handler_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    gossip_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
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
        config: BroadcastConfig,
        multicast_address: Address,
        membership_service: Arc<RwLock<MembershipService>>,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> BroadcastService {
        let (s, r): (Sender<DiscoveryMessage>, Receiver<DiscoveryMessage>) =
            crossbeam_channel::unbounded();

        let l = event_loop.clone();

        let gossip = Arc::new(GossipProtocol::new(
            config,
            membership_service,
            messaging_service,
            event_loop.clone(),
        ));

        BroadcastService {
            multicast_address,
            sender_thread: Arc::new(Mutex::new(Option::None)),
            handler_thread: Arc::new(Mutex::new(Option::None)),
            gossip_thread: Arc::new(Mutex::new(Option::None)),
            sender_channel: s,
            receiver_channel: r,
            gossip,
            event_loop,
        }
    }

    pub fn start(&self) -> Result<(), &str> {
        let multi_addr = self.multicast_address.ip;
        let multi_port = self.multicast_address.port;

        let multi_sock_addr = SockAddr::from(SocketAddrV4::new(multi_addr, multi_port));

        let socket_send = self.build_socket_send(&multi_sock_addr)?;
        let socket_receive = self.build_socket_receive(&multi_addr, multi_port)?;

        let sender_thread = self.start_sending(socket_send)?;
        let handler_thread = self.start_listening(socket_receive)?;
        let gossip_thread = self.start_gossip()?;

        //set thread handler to service. Service is the thread owner
        self.sender_thread.lock().unwrap().replace(sender_thread);
        self.handler_thread.lock().unwrap().replace(handler_thread);
        //println!("[BroadcastService]: Started");

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
            //println!("[BroadcastService]: Started sending multicast messages");
            for msg in receiver_channel_.iter() {
                let msg_bytes = serialize::to_bytes(&msg).unwrap();

                match socket.send(msg_bytes.as_slice()) {
                    Ok(_) => {
                        //println!("[BroadcastService]: Sent message to multicast group: OK");
                    }
                    Err(_) => {}//eprintln!("[BroadcastService]: Sent message to multicast group: ERR"),
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
                Err(_) => {}//println!("[BroadcastService]: Read message via multicast: ERR"),
                _ => {}
            }
        });

        Ok(thread)
    }

    fn start_gossip(&self) -> Result<std::thread::JoinHandle<()>, &str> {
        let gossp_ = self.gossip.clone();

        let thread = std::thread::spawn(move || loop {
            gossp_.start();
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

    pub fn add_broadcast_listener<F>(&self, f: F) -> Result<(), Box<()>>
    where
        F: Fn(Arc<BroadcastMessage>) -> () + 'static + Send + Sync,
    {
        match self.gossip.add_listener(f) {
            Ok(_) => Ok(()),
            Err(_) => Err(Box::new(())),
        }
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
    config: BroadcastConfig,
    listeners: RwLock<Vec<Box<Fn(Arc<BroadcastMessage>) -> () + 'static + Send + Sync>>>,
    send_buffer: chashmap::CHashMap<Uuid, Arc<RwLock<BufferedBroadcast>>>,
    keep_buffer: chashmap::CHashMap<Uuid, Arc<RwLock<BufferedBroadcast>>>,
    send_keys: RwLock<Vec<Uuid>>,
    keep_keys: RwLock<Vec<Uuid>>,
    membership_service: Arc<RwLock<MembershipService>>,
    messaging_service: Arc<RwLock<MessagingService>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl GossipProtocol {
    fn new(
        config: BroadcastConfig,
        membership_service: Arc<RwLock<MembershipService>>,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> GossipProtocol {
        GossipProtocol {
            config,
            listeners: RwLock::new(Vec::new()),
            send_buffer: chashmap::CHashMap::new(),
            keep_buffer: chashmap::CHashMap::new(),
            send_keys: RwLock::new(Vec::new()),
            keep_keys: RwLock::new(Vec::new()),
            membership_service,
            messaging_service,
            event_loop,
        }
    }

    fn start(&self) {
        let mut rng = &mut rand::thread_rng();

        loop {
            println!("[BroadcastService] SEND_BUFFER {:?}", self.send_buffer);
            println!("[BroadcastService] KEEP BUFFER {:?}", self.keep_buffer);
            let buffered_broadcast = self.choose_message_to_broadcast(&mut rng);

            let peer_count = self.membership_service.read().unwrap().get_member_count();

            if peer_count != 0 {
                if let Some(mut msg) = buffered_broadcast {
                    {
                        let payload = &msg.read().unwrap().payload;
                        let bytes_to_broadcast = serialize::to_bytes(payload).unwrap();
                        let peers = self.choose_peers_to_broadcast(rng);

                        self.do_broadcast(bytes_to_broadcast, peers);
                    }

                    // decrease ttl of the current message
                    msg.write().unwrap().rounds -= 1_i32;
                }

                //                println!("Before moving");
                self.move_to_keep_buffer();
            }

            //            println!("Before keeping");
            self.remove_from_keep_buffer();

            std::thread::sleep(Duration::from_millis(self.config.rate_ms))
        }
    }

    //put message into buffer
    fn send_new_broadcast(&self, payload: Vec<u8>) {
        let broadcast_payload = BroadcastMessage {
            id: uuid::Uuid::new_v4(),
            payload,
        };

        self.add_to_send_buffer(broadcast_payload);
    }

    fn handle_received_broadcast(&self, payload: BroadcastMessage) {
        if !self.keep_buffer.contains_key(&payload.id)
            && !self.send_buffer.contains_key(&payload.id)
        {
            self.notify_listeners(payload.clone());
            self.add_to_send_buffer(payload);
        }
    }

    fn add_to_send_buffer(&self, payload: BroadcastMessage) {
        let key = payload.id.clone();
        let nodes = self.membership_service.read().unwrap().get_member_count() as isize as f32;

        let buffered_message = BufferedBroadcast {
            rounds: get_rounds_count(nodes, self.config.rate_ms as f32),
            send: true,
            payload,
        };

        //        println!("New broadcast {:?}", buffered_message);

        self.send_buffer
            .insert_new(key.clone(), Arc::new(RwLock::new(buffered_message)));
        if self.send_buffer.get(&key).is_some() {
            self.send_keys.write().unwrap().push(key)
        }
    }

    fn choose_peers_to_broadcast(&self, rng: &mut ThreadRng) -> Vec<NodeMeta> {
        self.membership_service
            .read()
            .unwrap()
            .get_members()
            .choose_multiple(rng, self.config.fanout as usize)
            .cloned()
            .collect()
    }

    fn choose_message_to_broadcast(
        &self,
        rng: &mut ThreadRng,
    ) -> Option<ReadGuard<Uuid, Arc<RwLock<BufferedBroadcast>>>> {
        self.send_keys
            .read()
            .unwrap()
            .choose(rng)
            .and_then(|key| self.send_buffer.get(key))
    }

    fn do_broadcast(&self, bytes: Vec<u8>, peers: Vec<NodeMeta>) {
        for peer in peers.iter() {
            self.messaging_service.read().unwrap().send_to_member_type(
                bytes.clone(),
                peer,
                MessageType::Broadcast,
            );
        }
    }

    fn move_to_keep_buffer(&self) {
        for key in self.send_keys.read().unwrap().iter() {
            if let Some(br) = self.send_buffer.get(key) {
                if br.read().unwrap().rounds < 0 {
                    br.write().unwrap().send = false;
                    self.keep_buffer.insert(key.clone(), br.clone());
                    self.keep_keys.write().unwrap().push(key.clone())
                }
            }
        }

        self.send_buffer
            .retain(|_, value| value.read().unwrap().send);
        self.send_keys
            .write()
            .unwrap()
            .retain(|key| self.send_buffer.contains_key(key));
    }

    fn remove_from_keep_buffer(&self) {
        for key in self.keep_keys.read().unwrap().iter() {
            if let Some(br) = self.keep_buffer.get(key) {
                br.write().unwrap().rounds -= 1_i32;
            }
        }
        self.keep_buffer
            .retain(|_, value| value.read().unwrap().rounds > -self.config.message_keep);
        self.keep_keys
            .write()
            .unwrap()
            .retain(|key| self.keep_buffer.contains_key(key));
    }

    pub fn add_listener<F>(&self, f: F) -> Result<(), ()>
    where
        F: Fn(Arc<BroadcastMessage>) -> () + 'static + Send + Sync,
    {
        self.listeners.write().unwrap().push(Box::new(f));
        Ok(())
    }

    fn notify_listeners(&self, payload: BroadcastMessage) {
        let payload = Arc::new(payload);
        for listener in self.listeners.read().unwrap().iter() {
            listener(payload.clone());
        }
    }
}

fn get_rounds_count(nodes: f32, fanout: f32) -> i32 {
    if nodes <= 1.0 {
        return 1_i32; // set one round if there is no nodes
    }

    let prob = 0.99_f32;

    let x: f32 = nodes * prob / (1_f32 - prob);
    let round_count: f32 = 2_f32 * x.ln() / fanout;

    round_count.floor() as i64 as i32
}

#[derive(Debug)]
struct BufferedBroadcast {
    rounds: i32,
    send: bool,
    payload: BroadcastMessage,
}
