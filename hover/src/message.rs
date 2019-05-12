extern crate chashmap;
extern crate socket2;
extern crate uuid;

use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chashmap::CHashMap;
use crossbeam_channel::{Receiver, Sender};
use socket2::{Domain, SockAddr, Socket, Type};

use crate::common::{Address, BroadcastMessage, Message, MessageType, NodeMeta, ProbeReqPayload};
use crate::events::Event::{BroadcastIn, ProbeIn, ProbeReqIn};
use crate::events::{Event, EventListener, EventLoop};
use crate::membership::MembershipService;
use crate::serialize;

use self::uuid::Uuid;

pub struct MessageDispatcher {
    listeners: Vec<Box<Fn(Arc<Message>) -> () + 'static + Send + Sync>>,
    resp_callbacks: RwLock<CHashMap<Uuid, Sender<Arc<Message>>>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl MessageDispatcher {
    pub fn new(event_loop: Arc<RwLock<EventLoop>>) -> MessageDispatcher {
        MessageDispatcher {
            listeners: Vec::new(),
            resp_callbacks: RwLock::new(CHashMap::new()),
            event_loop,
        }
    }

    pub fn add_msg_listener<F>(&mut self, f: F) -> Result<(), ()>
    where
        F: Fn(Arc<Message>) -> () + 'static + Send + Sync,
    {
        self.listeners.push(Box::new(f));
        Ok(())
    }

    fn add_resp_callback(&self, msg_id: Uuid, sender: Sender<Arc<Message>>) {
        match self.resp_callbacks.write().unwrap().insert(msg_id, sender) {
            Some(_) => {
                println!("[MessageDispatcher]: overrides a resp_callback!");
            }
            None => {}
        }
    }

    fn remove_resp_callback(&self, msg_id: Uuid) {
        self.resp_callbacks.write().unwrap().remove(&msg_id);
    }

    fn handle_in_message(&self, msg: Arc<Message>) {
        match msg.msg_type {
            MessageType::Request => self.handle_request(msg),
            MessageType::Response => self.handle_response(msg),
            MessageType::Probe => self.send_event(self.build_probe_in_event(msg)),
            MessageType::ProbeReq => self.send_event(self.build_probe_req_in_event(msg)),
            MessageType::Broadcast => self.send_event(self.build_broadcast_in_event(msg)),
        }
    }

    fn handle_request(&self, msg: Arc<Message>) {
        for listener in self.listeners.iter() {
            listener(msg.clone())
        }
    }

    fn handle_response(&self, msg: Arc<Message>) {
        match self.resp_callbacks.write().unwrap().remove(&msg.cor_id) {
            Some(sender) => {
                sender.send(msg);
            }
            None => {}
        }
    }

    fn send_event(&self, event: Event) {
        self.event_loop.read().unwrap().post_event(event);
    }

    fn build_probe_in_event(&self, msg: Arc<Message>) -> Event {
        ProbeIn {
            cor_id: msg.cor_id.clone(),
            return_address: msg.return_address.clone(),
        }
    }

    fn build_probe_req_in_event(&self, msg: Arc<Message>) -> Event {
        let probe_payload: ProbeReqPayload =
            serialize::from_bytes(msg.payload.clone().as_slice()).unwrap();

        ProbeReqIn {
            cor_id: msg.cor_id.clone(),
            probe_node: probe_payload.node,
            return_address: msg.return_address.clone(),
        }
    }

    fn build_broadcast_in_event(&self, msg: Arc<Message>) -> Event {
        let broadcast_payload: BroadcastMessage =
            serialize::from_bytes(msg.payload.clone().as_slice()).unwrap();

        BroadcastIn {
            payload: broadcast_payload,
        }
    }
}

impl EventListener for MessageDispatcher {
    fn on_event(&self, event: Event) {
        match event {
            Event::MessageIn { msg } => self.handle_in_message(msg),
            _ => {}
        }
    }
}

/**Service for sending messages across cluster.*/
pub struct MessagingService {
    local_node: NodeMeta,
    message_dispatcher: Arc<RwLock<MessageDispatcher>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl MessagingService {
    pub fn new(
        local_node: NodeMeta,
        message_dispatcher: Arc<RwLock<MessageDispatcher>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> MessagingService {
        MessagingService {
            local_node,
            message_dispatcher,
            event_loop,
        }
    }

    /**public*/
    pub fn reply(
        &self,
        msg_id: Uuid,
        payload: Vec<u8>,
        address: Address,
    ) -> Result<(), Box<Error>> {
        let msg = Message {
            cor_id: msg_id,
            return_address: self.local_node.addr.clone(),
            msg_type: MessageType::Response,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send(msg_bytes, &address)?;

        Ok(())
    }

    /**public*/
    pub fn send_to_address(&self, payload: Vec<u8>, address: Address) -> Result<(), Box<Error>> {
        self.send_to_address_type(payload, address, MessageType::Request)
    }

    /**public*/
    pub fn send_to_address_type(
        &self,
        payload: Vec<u8>,
        address: Address,
        msg_type: MessageType,
    ) -> Result<(), Box<Error>> {
        let msg = Message {
            cor_id: gen_msg_id(),
            return_address: self.local_node.addr.clone(),
            msg_type,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send(msg_bytes, &address)?;

        Ok(())
    }

    /**public*/
    pub fn send_to_member(&self, payload: Vec<u8>, member: &NodeMeta) -> Result<(), Box<Error>> {
        self.send_to_member_type(payload, member, MessageType::Request)
    }

    /**public*/
    pub fn send_to_member_type(
        &self,
        payload: Vec<u8>,
        member: &NodeMeta,
        msg_type: MessageType,
    ) -> Result<(), Box<Error>> {
        let msg = Message {
            cor_id: gen_msg_id(),
            return_address: self.local_node.addr.clone(),
            msg_type,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send(msg_bytes, &member.addr)?;

        Ok(())
    }

    /**public*/
    pub fn send_to_address_receive(
        &self,
        payload: Vec<u8>,
        address: Address,
        timeout: Duration,
    ) -> Result<Arc<Message>, Box<Error>> {
        self.send_to_address_receive_type(payload, address, MessageType::Request, timeout)
    }

    /**public*/
    pub fn send_to_address_receive_type(
        &self,
        payload: Vec<u8>,
        address: Address,
        msg_type: MessageType,
        timeout: Duration,
    ) -> Result<Arc<Message>, Box<Error>> {
        let correlation_id = gen_msg_id();
        let msg = Message {
            cor_id: correlation_id,
            return_address: self.local_node.addr.clone(),
            msg_type,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send_receive(correlation_id, msg_bytes, &address, timeout)
    }

    /**public*/
    pub fn send_to_member_receive(
        &self,
        payload: Vec<u8>,
        member: &NodeMeta,
        timeout: Duration,
    ) -> Result<Arc<Message>, Box<Error>> {
        self.send_to_member_receive_type(payload, member, MessageType::Request, timeout)
    }

    /**public*/
    pub fn send_to_member_receive_type(
        &self,
        payload: Vec<u8>,
        member: &NodeMeta,
        msg_type: MessageType,
        timeout: Duration,
    ) -> Result<Arc<Message>, Box<Error>> {
        let correlation_id = gen_msg_id();
        let msg = Message {
            cor_id: correlation_id,
            return_address: self.local_node.addr.clone(),
            msg_type,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send_receive(correlation_id, msg_bytes, &member.addr, timeout)
    }

    /**public*/
    pub fn broadcast(&self, bytes: Vec<u8>) -> Result<(), Box<Error>> {
        let event = Event::BroadcastOut { payload: bytes };

        self.event_loop.read().unwrap().post_event(event)
    }

    fn do_send(&self, mut bytes: Vec<u8>, addr: &Address) -> Result<(), Box<Error>> {
        match TcpStream::connect((addr.ip, addr.port)) {
            Ok(mut stream) => match stream.write_all(bytes.as_mut_slice()) {
                Ok(_) => Ok(()),
                Err(err) => Err(Box::new(err)),
            },
            Err(err) => Err(Box::new(err)),
        }
    }

    fn do_send_receive(
        &self,
        correlation_id: Uuid,
        mut bytes: Vec<u8>,
        addr: &Address,
        timeout: Duration,
    ) -> Result<Arc<Message>, Box<Error>> {
        //create channel between receiver and current thread
        let (s, r): (Sender<Arc<Message>>, Receiver<Arc<Message>>) = crossbeam_channel::bounded(1);

        //add message listener for given correlation id
        self.message_dispatcher
            .write()
            .unwrap()
            .add_resp_callback(correlation_id, s);

        match self.do_send(bytes, &addr) {
            Err(err) => {
                eprintln!("[MessageSercive]: Error while sending a message!");
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_resp_callback(correlation_id);
                return Err(err);
            }
            Ok(_) => {}
        }

        //block until received response
        return match r.recv_timeout(timeout) {
            Ok(response) => {
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_resp_callback(correlation_id);
                Ok(response)
            }
            Err(err) => {
                eprintln!("[MessageService]: Error while waiting for the response!");
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_resp_callback(correlation_id);
                Err(Box::new(err))
            }
        };
    }
}

fn gen_msg_id() -> Uuid {
    Uuid::new_v4()
}
