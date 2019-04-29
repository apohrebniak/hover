extern crate socket2;

use std::collections::HashSet;

use crate::cluster::Member;
use crate::common::{Address, NodeMeta};
use crate::events::{Event, EventListener};
use crate::service::Service;

/**Service that allows to retrieve info about cluster members*/
pub struct MembershipService {}

impl MembershipService {
    pub fn new() -> MembershipService {
        MembershipService {}
    }

    pub fn get_members(&self) -> HashSet<Member> {
        HashSet::new() //TODO
    }

    pub fn get_member_by_id(&self, member_id: &str) -> Option<Member> {
        None
    }

    pub fn get_member_by_address(&self, address: Address) -> Option<Member> {
        None
    }

    fn handle_joined_node(&self, node: NodeMeta) {
        println!("[MembershipService]: Handled joined node: {:?}", node);
    }

    fn handle_left_node(&self, node: NodeMeta) {
        println!("[MembershipService]: Handled left node: {:?}", node);
    }
}

impl Service for MembershipService {
    fn start(&self) {
        println!("[MembershipService]: Membership service started");
    }
}

impl EventListener for MembershipService {
    fn on_event(&self, event: Event) {
        match event {
            Event::JoinIn { node_meta } => {
                self.handle_joined_node(node_meta);
            }
            Event::LeaveIn { node_meta } => {
                self.handle_left_node(node_meta);
            }
            _ => {}
        }
    }
}

// react on ping
//react on ping-req
//add message service
