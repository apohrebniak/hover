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