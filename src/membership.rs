extern crate rand;
extern crate socket2;

use std::collections::HashSet;

use self::rand::seq::SliceRandom;
use crate::common::{Address, Message, MessageType, NodeMeta, ProbeReqPayload};
use crate::events::Event::{MemberAdded, MemberLeft};
use crate::events::{Event, EventListener, EventLoop};
use crate::message::MessagingService;
use crate::serialize;

use chashmap::CHashMap;
use core::borrow::Borrow;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use uuid::Uuid;

/**Service that allows to retrieve info about cluster members*/
pub struct MembershipService {
    local_node_meta: NodeMeta,
    messaging_service: Arc<RwLock<MessagingService>>,
    swim: Arc<SwimProtocol>,
    swim_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl MembershipService {
    pub fn new(
        local_node_meta: NodeMeta,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> MembershipService {
        let swim = SwimProtocol::new(
            local_node_meta.clone(),
            messaging_service.clone(),
            event_loop,
        );

        MembershipService {
            local_node_meta,
            messaging_service,
            swim: Arc::new(swim),
            swim_thread: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) {
        let swim_ = self.swim.clone();

        let thread = std::thread::spawn(move || {
            swim_.start();
        });

        self.swim_thread.lock().unwrap().replace(thread);

        println!("[MembershipService]: Membership service started");
    }

    /**Returns the full copy of the current members state*/
    pub fn get_members(&self) -> Vec<NodeMeta> {
        self.swim.members.clone().read().unwrap().clone()
    }

    pub fn get_member_by_id(&self, member_id: &Uuid) -> Option<NodeMeta> {
        self.swim
            .members
            .read()
            .unwrap()
            .iter()
            .find(|n| n.id == *member_id)
            .map(|n| n.clone())
    }

    pub fn get_member_by_address(&self, address: &Address) -> Option<NodeMeta> {
        self.swim
            .members
            .read()
            .unwrap()
            .iter()
            .find(|n| n.addr == *address)
            .map(|n| n.clone())
    }

    pub fn get_member_count(&self) -> usize {
        self.swim.members.read().unwrap().len()
    }

    fn handle_joined_node(&self, node: NodeMeta) {
        println!("[MembershipService]: Handled joined node: {:?}", node);
        self.swim.add_member(node);
    }

    fn handle_left_node(&self, node: NodeMeta) {
        println!("[MembershipService]: Handled left node: {:?}", node);
        self.swim.remove_member(node);
    }

    fn handle_probe(&self, cor_id: Uuid, return_addr: Address) {
        self.messaging_service
            .read()
            .unwrap()
            .reply(cor_id, Vec::new(), return_addr);
    }

    fn handle_probe_req(&self, cor_id: Uuid, probe_node: NodeMeta, return_addr: Address) {
        match self.swim.probe_member(&probe_node) {
            Ok(_) => {
                self.messaging_service
                    .read()
                    .unwrap()
                    .reply(cor_id, Vec::new(), return_addr);
            }
            _ => {}
        }
    }
}

impl EventListener for MembershipService {
    fn on_event(&self, event: Event) {
        match event {
            Event::JoinIn { node_meta } => {
                self.handle_joined_node(node_meta);
            }
            Event::LeftIn { node_meta } => {
                self.handle_left_node(node_meta);
            }
            Event::ProbeIn {
                cor_id,
                return_address,
            } => {
                self.handle_probe(cor_id, return_address);
            }
            Event::ProbeReqIn {
                cor_id,
                probe_node,
                return_address,
            } => {
                self.handle_probe_req(cor_id, probe_node, return_address);
            }
            _ => {}
        }
    }
}

/**SWIM protocol logic and process*/
struct SwimProtocol {
    local_node_meta: NodeMeta,
    members: Arc<RwLock<Vec<NodeMeta>>>,
    messaging_service: Arc<RwLock<MessagingService>>,
    // left members queue
    event_loop: Arc<RwLock<EventLoop>>,
}

impl SwimProtocol {
    fn new(
        local_node_meta: NodeMeta,
        messaging_service: Arc<RwLock<MessagingService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> SwimProtocol {
        SwimProtocol {
            local_node_meta,
            members: Arc::new(RwLock::new(Vec::new())),
            messaging_service,
            event_loop,
        }
    }

    fn start(&self) {
        let mut rng = &mut rand::thread_rng();

        loop {
            println!(
                "[MembershipService]: Available members: {:?}",
                self.members.read().unwrap().as_slice()
            );

            let members_ = self.members.read().unwrap().clone();

            let member_to_probe: Option<NodeMeta> = members_.choose(rng).map(|n| n.clone());
            if let Some(member_to_probe) = member_to_probe {
                if let Err(_) = self.probe_member(&member_to_probe) {
                    let other_members: Vec<&NodeMeta> = members_.choose_multiple(rng, 2).collect(); //TODO: config

                    let is_available = other_members
                        .into_iter()
                        .map(|member| self.probe_request_member(&member_to_probe, member))
                        .find_map(|result| result.ok())
                        .is_some();

                    if !is_available {
                        self.remove_member(member_to_probe);
                    }
                }
            }

            std::thread::sleep_ms(5000); //TODO: config
        }
    }

    fn probe_member(&self, member_to_probe: &NodeMeta) -> Result<(), Box<Error>> {
        self.messaging_service
            .read()
            .unwrap()
            .send_to_member_receive_type(
                Vec::new(),
                member_to_probe,
                MessageType::Probe,
                Duration::new(1, 0),
            ) //TODO: config
            .map(|_| ())
    }

    fn probe_request_member(
        &self,
        member_to_probe: &NodeMeta,
        member: &NodeMeta,
    ) -> Result<(), Box<Error>> {
        let payload = ProbeReqPayload {
            node: member_to_probe.clone(),
        };

        let payload_bytes = serialize::to_bytes(&payload).unwrap();

        self.messaging_service
            .read()
            .unwrap()
            .send_to_member_receive_type(
                payload_bytes,
                member,
                MessageType::ProbeReq,
                Duration::new(1, 0),
            ) //TODO: config
            .map(|_| ())
    }

    fn add_member(&self, node: NodeMeta) {
        if self.local_node_meta.id != node.id && !self.members.read().unwrap().contains(&node) {
            println!("[MembershipService]: Added node to cluster {:?}", &node);
            self.members.write().unwrap().push(node.clone());
            self.event_loop
                .read()
                .unwrap()
                .post_event(MemberAdded { node_meta: node });
        }
    }

    fn remove_member(&self, node: NodeMeta) {
        if self.members.read().unwrap().contains(&&node) {
            println!("[MembershipService]: Removed node from cluster {:?}", &node);
            self.members.write().unwrap().retain(|x| x != &node);
            self.event_loop
                .read()
                .unwrap()
                .post_event(MemberLeft { node_meta: node });
        }
    }
}
