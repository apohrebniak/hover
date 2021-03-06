use crate::common::NodeMeta;
use crate::events::{Event, EventListener, EventLoop};
use crate::membership::MembershipService;

use crate::config::DiscoveryConfig;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

/**Send Join and Leave events periodicaly*/
pub struct DiscoveryProvider {
    local_node_meta: NodeMeta,
    config: DiscoveryConfig,
    worker_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    membership_service: Arc<RwLock<MembershipService>>,
    event_loop: Arc<RwLock<EventLoop>>,
}

impl DiscoveryProvider {
    pub fn new(
        local_node_meta: NodeMeta,
        config: DiscoveryConfig,
        membership_service: Arc<RwLock<MembershipService>>,
        event_loop: Arc<RwLock<EventLoop>>,
    ) -> DiscoveryProvider {
        DiscoveryProvider {
            local_node_meta,
            config,
            worker_thread: Arc::new(Mutex::new(None)),
            membership_service,
            event_loop,
        }
    }

    pub fn start(&self) {
        let loop_ = self.event_loop.clone();
        let local_join_event = Event::JoinOut {
            node_meta: self.local_node_meta.clone(),
        };
        let rate = self.config.rate_ms;

        let thread = std::thread::spawn(move || loop {
            loop_
                .write()
                .unwrap()
                .post_event(local_join_event.clone())
                .unwrap();

            std::thread::sleep(Duration::from_millis(rate))
        });

        self.worker_thread.lock().unwrap().replace(thread);
        println!("[DiscoveryProvider]: Started")
    }
}

impl EventListener for DiscoveryProvider {
    fn on_event(&self, event: Event) {
        if let Event::MemberLeft { node_meta } = event {
            self.event_loop
                .read()
                .unwrap()
                .post_event(Event::LeftOut { node_meta });
        }
    }
}
