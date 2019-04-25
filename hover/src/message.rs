extern crate socket2;
extern crate uuid;

use std::collections::HashSet;

use crate::cluster::Member;
use crate::common::{Address, Message, MessageType, NodeMeta};
use crate::events::{Event, EventListener};
use crate::serialize;
use crate::service::Service;

use self::uuid::Uuid;
use crate::service::cluster_service::MembershipService;
use crossbeam_channel::{Receiver, Sender};
use socket2::{Domain, SockAddr, Socket, Type};
use std::error::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct MessageDispatcher {
    listeners: Vec<Box<Fn(Arc<Message>) -> () + 'static + Send + Sync>>,
}

impl MessageDispatcher {
    pub fn new() -> MessageDispatcher {
        MessageDispatcher {
            listeners: Vec::new(),
        }
    }

    pub fn add_msg_listener<F>(&mut self, f: F) -> Result<(), ()>
    where
        F: Fn(Arc<Message>) -> () + 'static + Send + Sync,
    {
        self.listeners.push(Box::new(f));
        Ok(())
    }

    fn add_msg_receiver(&self, msg_id: Uuid, sender: Sender<Message>) {
        println!("RECEIVER ADDED");
    }

    fn remove_msg_receiver(&self, msg_id: Uuid) {
        println!("RECEIVER REMOVED")
    }

    //TODO: implement
    pub fn subscribe_for_topic(&mut self) -> Result<(), ()> {
        Ok(())
    }

    fn handle_in_message(&self, msg: Arc<Message>) {
        for listener in self.listeners.iter() {
            listener(msg.clone())
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
    membership_service: Arc<RwLock<MembershipService>>,
    message_dispatcher: Arc<RwLock<MessageDispatcher>>,
}

impl MessagingService {
    pub fn new(
        local_node: NodeMeta,
        membership_service: Arc<RwLock<MembershipService>>,
        message_dispatcher: Arc<RwLock<MessageDispatcher>>,
    ) -> MessagingService {
        MessagingService {
            local_node,
            membership_service,
            message_dispatcher,
        }
    }

    /**public*/
    pub fn send_to_address(&self, payload: Vec<u8>, address: Address) -> Result<(), Box<Error>> {
        let msg = Message {
            corId: gen_msg_id(),
            return_address: Some(self.local_node.addr.clone()),
            msg_type: MessageType::REQUEST,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        self.do_send(msg_bytes, &address)?;

        Ok(())
    }

    /**public*/
    pub fn send_to_member(&self, payload: Vec<u8>, member: Member) -> Result<(), Box<Error>> {
        let msg = Message {
            corId: gen_msg_id(),
            return_address: Some(self.local_node.addr.clone()),
            msg_type: MessageType::REQUEST,
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
    ) -> Result<Message, Box<Error>> {
        let correlation_id = gen_msg_id();
        let msg = Message {
            corId: correlation_id,
            return_address: Some(self.local_node.addr.clone()),
            msg_type: MessageType::REQUEST,
            payload,
        };
        let mut msg_bytes = serialize::to_bytes(&msg).unwrap();

        //create channel between receiver and current thread
        let (s, r): (Sender<Message>, Receiver<Message>) = crossbeam_channel::bounded(1);

        //add message listener for given correlation id
        self.message_dispatcher
            .write()
            .unwrap()
            .add_msg_receiver(correlation_id, s);

        match self.do_send(msg_bytes, &address) {
            Err(err) => {
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_msg_receiver(correlation_id);
                return Err(err);
            }
            Ok(_) => {}
        }

        //block until received response
        match r.recv_timeout(timeout) {
            Ok(response) => {
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_msg_receiver(correlation_id);
                Ok(response)
            }
            Err(err) => {
                self.message_dispatcher
                    .write()
                    .unwrap()
                    .remove_msg_receiver(correlation_id);
                Err(Box::new(err))
            }
        }
    }

    /**public*/
    pub fn send_to_member_receive(&self, msg: Vec<u8>, member: Member) -> Result<Message, &str> {
        Ok(Message {
            corId: gen_msg_id(),
            msg_type: MessageType::REQUEST,
            payload: Vec::new(),
            return_address: None,
        })
    }

    /**public*/
    pub fn broadcast(&self, msg: Vec<u8>) -> Result<(), &str> {
        Ok(())
    }

    /**public*/
    //TODO: consider subscription topics
    pub fn multicast_to_addresses(
        &self,
        msg: Message,
        addresses: HashSet<Address>,
    ) -> Result<(), &str> {
        Ok(())
    }

    /**public*/
    pub fn multicast_to_members(&self, msg: Message, members: HashSet<Member>) -> Result<(), &str> {
        Ok(())
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
}

impl Service for MessagingService {
    fn start(&self) {
        println!("[MessagingService]: Messaging service started")
    }
}

fn gen_msg_id() -> Uuid {
    Uuid::new_v4()
}
