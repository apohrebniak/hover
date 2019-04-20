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
use socket2::{Domain, SockAddr, Socket, Type};
use std::error::Error;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

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
}

impl MessagingService {
    pub fn new(
        local_node: NodeMeta,
        membership_service: Arc<RwLock<MembershipService>>,
    ) -> MessagingService {
        MessagingService {
            local_node,
            membership_service,
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

        self.do_send(msg_bytes, &address)
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

        self.do_send(msg_bytes, &member.addr)
    }

    /**public*/
    pub fn send_to_address_receive(&self, msg: Vec<u8>, address: Address) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {
            corId: gen_msg_id(),
            msg_type: MessageType::REQUEST,
            payload: Vec::new(),
            return_address: None,
        })
    }

    /**public*/
    pub fn send_to_member_receive(&self, msg: Vec<u8>, member: Member) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {
            corId: gen_msg_id(),
            msg_type: MessageType::REQUEST,
            payload: Vec::new(),
            return_address: None,
        })
    }

    /**public*/
    pub fn broadcast(&self, msg: Vec<u8>) -> Result<(), &str> {
        dbg!("Broadcasted");
        Ok(())
    }

    /**public*/
    //TODO: consider subscription topics
    pub fn multicast_to_addresses(
        &self,
        msg: Message,
        addresses: HashSet<Address>,
    ) -> Result<(), &str> {
        dbg!("multicast to addresses");
        Ok(())
    }

    /**public*/
    pub fn multicast_to_members(&self, msg: Message, members: HashSet<Member>) -> Result<(), &str> {
        dbg!("multicast to members");
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
        println!("Messaging service started")
    }
}

fn gen_msg_id() -> Uuid {
    Uuid::new_v4()
}
