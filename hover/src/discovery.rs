use crate::common::NodeMeta;
use crate::events::{Event, EventListener, EventLoop};
use crate::service::membership::MembershipService;
use crate::service::Service;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

/**Send Join and Leave events periodicaly*/
pub struct DiscoveryProvider {
    local_node_meta: NodeMeta,
    worker_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    membership_service: Arc<RwLock<MembershipService>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl DiscoveryProvider {
    pub fn new(
        local_node_meta: NodeMeta,
        membership_service: Arc<RwLock<MembershipService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> DiscoveryProvider {
        DiscoveryProvider {
            local_node_meta,
            worker_thread: Arc::new(Mutex::new(None)),
            membership_service,
            event_loop,
        }
    }

    pub fn start_inner(&self) {
        let loop_ = self.event_loop.clone();
        let local_join_event = Event::JoinOut {
            node_meta: self.local_node_meta.clone(),
        };

        let thread = std::thread::spawn(move || loop {
            loop_
                .write()
                .unwrap()
                .post_event(local_join_event.clone())
                .unwrap();

            std::thread::sleep_ms(3000);
        });

        self.worker_thread.lock().unwrap().replace(thread);
        println!("[DiscoveryProvider]: Started")
    }
}

impl Service for DiscoveryProvider {
    fn start(&self) {
        self.start_inner();
    }
}
