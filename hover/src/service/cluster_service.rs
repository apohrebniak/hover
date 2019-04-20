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

    fn handle_discovered_node(&self, node: NodeMeta) {
        println!("Handle discovered node: {:?}", node);
    }
}

impl Service for MembershipService {
    fn start(&self) {
        dbg!("Cluster service started");
    }
}

impl EventListener for MembershipService {
    fn on_event(&self, event: Event) {
        match event {
            Event::DiscoveryIn { node_meta } => {
                self.handle_discovered_node(node_meta);
            }
            Event::Empty => {
                dbg!("Handled an empty event");
            }
            _ => {}
        }
    }
}
