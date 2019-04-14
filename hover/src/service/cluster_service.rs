extern crate socket2;

use std::collections::HashSet;

use crate::cluster::Member;
use crate::common::Address;
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
}

impl Service for ClusterService {
    fn start(&self) {
        dbg!("Cluster service started");
    }
}

impl EventListener for ClusterService {
    fn on_event(&self, event: Event) {
        println!("DAMN BOIIIIII");
    }
}
