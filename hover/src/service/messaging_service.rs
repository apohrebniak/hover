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

/**Service for sending messages across cluster.*/
pub struct MessagingService {}

impl MessagingService {
    pub fn new() -> MessagingService {
        MessagingService {}
    }

    pub fn send_to_address(&self, msg: Message, address: Address) -> Result<(), &str> {
        dbg!("Sent message to address");
        Ok(())
    }

    pub fn send_to_member(&self, msg: Message, member: Member) -> Result<(), &str> {
        dbg!("Send message to member");
        Ok(())
    }

    pub fn send_to_address_receive(&self, msg: Message, address: Address) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {})
    }

    pub fn send_to_member_receive(&self, msg: Message, member: Member) -> Result<Message, &str> {
        dbg!("send and receive to address");
        Ok(Message {})
    }

    pub fn broadcast(&self, msg: Message) -> Result<(), &str> {
        dbg!("Broadcasted");
        Ok(())
    }

    //TODO: consider subscription topics
    pub fn multicast_to_addresses(
        &self,
        msg: Message,
        addresses: HashSet<Address>,
    ) -> Result<(), &str> {
        dbg!("multicast to addresses");
        Ok(())
    }

    pub fn multicast_to_members(&self, msg: Message, members: HashSet<Member>) -> Result<(), &str> {
        dbg!("multicast to members");
        Ok(())
    }
}

impl Service for MessagingService {
    fn start(&self) {
        println!("Messaging service started")
    }
}