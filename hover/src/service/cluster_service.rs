extern crate socket2;

use std::collections::HashSet;

use crate::cluster::Member;
use crate::common::{Address, NodeMeta};
use crate::events::{Event, EventListener};
use crate::service::Service;

/**Service that allows to retrieve info about cluster members*/
pub struct ClusterService {}

impl ClusterService {
    pub fn new() -> ClusterService {
        ClusterService {}
    }

    pub fn get_members() -> HashSet<Member> {
        HashSet::new() //TODO
    }

    pub fn get_member_by_id(member_id: &str) -> Option<Member> {
        None
    }

    pub fn get_member_by_address(address: Address) -> Option<Member> {
        None
    }

    fn handle_discovered_node(&self, node: NodeMeta) {
        println!("Handle discovered node: {:?}", node);
    }
}

impl Service for ClusterService {
    fn start(&self) {
        dbg!("Cluster service started");
    }
}

impl EventListener for ClusterService {
    fn on_event(&self, event: Event) {
        match event {
            Event::DiscoveryEvent { node_meta } => {
                self.handle_discovered_node(node_meta);
            }
            Event::Empty => {
                dbg!("Handled an empty event");
            }
        }
    }
}
