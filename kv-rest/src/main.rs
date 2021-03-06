extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::iter::FromIterator;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use futures::future;
use futures::future::Future;
use futures::stream::Stream;
use gotham::handler::HandlerFuture;
use gotham::handler::IntoHandlerError;
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use hover::common::{Address, NodeMeta};
use hover::events::EventListener;
use hyper::{Body, Response, StatusCode};
use mime::Mime;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

pub mod settings;

#[derive(Clone, StateData)]
struct HoverState {
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
    kv_nodes: Arc<RwLock<chashmap::CHashMap<Uuid, hover::common::Address>>>,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathStringExtractor {
    key: String,
}

#[derive(Serialize)]
struct KvMember {
    id: Uuid,
    http_address: Option<Address>,
    hover_node: NodeMeta,
}

#[derive(Deserialize)]
struct Input {
    value: String,
}

fn get_members(mut state: State) -> (State, Response<Body>) {
    let hover_state = HoverState::take_from(&mut state);
    let hover = hover_state.hover;
    let kv_nodes = hover_state.kv_nodes;

    let mut members = hover
        .read()
        .unwrap()
        .get_cluster_service()
        .map(|ms| ms.read().unwrap().get_members())
        .unwrap();

    members.retain(|nm| kv_nodes.read().unwrap().contains_key(&nm.id));

    let kv_members = members
        .into_iter()
        .map(|nm| KvMember {
            id: nm.id.clone(),
            http_address: kv_nodes.read().unwrap().get(&nm.id).map(|g| g.clone()),
            hover_node: nm,
        })
        .collect::<Vec<_>>();

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_JSON,
        serde_json::to_string(&kv_members).unwrap(),
    );
    (state, res)
}

fn get_kv_all(mut state: State) -> (State, Response<Body>) {
    let map = HoverState::take_from(&mut state).map;

    let local_map: HashMap<String, String> =
        HashMap::from_iter(map.read().unwrap().clone().into_iter());

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_JSON,
        serde_json::to_string(&local_map).unwrap(),
    );
    (state, res)
}

fn get_kv(mut state: State) -> (State, Response<Body>) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let map = HoverState::take_from(&mut state).map;

    let value = map.read().unwrap().get(&key).map(|v| v.clone());

    let res = match value {
        Some(v) => create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, v),
        None => create_empty_response(&state, StatusCode::NOT_FOUND),
    };

    (state, res)
}

fn body_handler<F>(mut state: State, f: F) -> Box<HandlerFuture>
where
    F: 'static + Send + Fn(Vec<u8>, &State) -> Response<Body>,
{
    let body = Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let body_content = valid_body.to_vec();
                let res = f(body_content, &mut state);
                future::ok((state, res))
            }
            Err(e) => return future::err((state, e.into_handler_error())),
        });
    Box::new(body)
}

fn post_kv(mut state: State) -> Box<HandlerFuture> {
    let key = PathStringExtractor::take_from(&mut state).key;
    let hover_state = HoverState::take_from(&mut state);
    let map = hover_state.map;
    let hover = hover_state.hover;

    body_handler(state, move |body_bytes, state| {
        let json: Result<Input, _> = serde_json::from_slice(body_bytes.as_slice());
        match json {
            Ok(input) => {
                map.read().unwrap().insert(key.clone(), input.value.clone());

                let event = MapEvent::Post {
                    key: key.clone(),
                    value: input.value.clone(),
                };
                let event = bincode::serialize(&event).unwrap();

                hover
                    .read()
                    .unwrap()
                    .get_messaging_service()
                    .unwrap()
                    .read()
                    .unwrap()
                    .broadcast(event);

                create_empty_response(state, StatusCode::OK)
            }
            Err(_) => create_empty_response(state, StatusCode::BAD_REQUEST),
        }
    })
}

fn delete_kv(mut state: State) -> (State, Response<Body>) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let hover_state = HoverState::take_from(&mut state);
    let map = hover_state.map;
    let hover = hover_state.hover;

    let removed_opt = map.read().unwrap().remove(&key);

    if let Some(_) = removed_opt {
        let event = MapEvent::Delete { key };
        let event = bincode::serialize(&event).unwrap();

        hover
            .read()
            .unwrap()
            .get_messaging_service()
            .unwrap()
            .read()
            .unwrap()
            .broadcast(event);
    };

    let res = create_empty_response(&state, StatusCode::OK);
    (state, res)
}

fn router(hover_state: HoverState) -> Router {
    let middleware = StateMiddleware::new(hover_state);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/members").to(get_members);
        route.get("/kv").to(get_kv_all);
        route
            .get("/kv/:key")
            .with_path_extractor::<PathStringExtractor>()
            .to(get_kv);
        route
            .post("/kv/:key")
            .with_path_extractor::<PathStringExtractor>()
            .to(post_kv);
        route
            .delete("/kv/:key")
            .with_path_extractor::<PathStringExtractor>()
            .to(delete_kv);
    })
}

pub fn main() {
    let settings = settings::Settings::new().unwrap();
    println!("Starting hover...");

    let hover = hover::Hover::default()
        .map(|h| Arc::new(RwLock::new(h)))
        .unwrap();

    let map = Arc::new(RwLock::new(chashmap::CHashMap::new()));

    let kv_nodes = Arc::new(RwLock::new(chashmap::CHashMap::new()));

    let kv_address = Arc::new(RwLock::new(hover::common::Address {
        ip: Ipv4Addr::from_str(settings.host.as_str()).unwrap(),
        port: settings.port,
    }));

    hover.write().unwrap().start();
    setup_hover(
        hover.clone(),
        map.clone(),
        kv_nodes.clone(),
        kv_address.clone(),
    );

    let hover_state = HoverState {
        hover,
        map,
        kv_nodes,
    };

    let router = router(hover_state);

    println!(
        "Listening for requests at http://{}:{}",
        settings.host, settings.port
    );
    gotham::start((Ipv4Addr::UNSPECIFIED, settings.port), router)
}

#[derive(Deserialize, Serialize)]
enum MapEvent {
    Post { key: String, value: String },
    Delete { key: String },
}

fn setup_hover(
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
    kv_nodes: Arc<RwLock<chashmap::CHashMap<Uuid, hover::common::Address>>>,
    kv_address: Arc<RwLock<hover::common::Address>>,
) {
    let hover_ = hover.clone();
    let map_ = map.clone();

    hover
        .write()
        .unwrap()
        .add_broadcast_listener(move |msg| {
            let event: MapEvent = bincode::deserialize(msg.payload.as_slice()).unwrap();

            match event {
                MapEvent::Post { key, value } => {
                    map_.read().unwrap().insert(key, value);
                }
                MapEvent::Delete { key } => {
                    map_.read().unwrap().remove(&key);
                }
            }
        })
        .unwrap();

    let member_added_listener = MapMemberAddedListener {
        hover: hover.clone(),
        map: map.clone(),
        kv_nodes: kv_nodes.clone(),
        kv_address: kv_address.clone(),
    };
    hover
        .read()
        .unwrap()
        .add_event_listener(member_added_listener);

    let map_ = map.clone();
    hover.write().unwrap().add_msg_listener(move |msg| {
        if let hover::common::MessageType::Request = msg.msg_type {
            let kv_msg: KvMessage = bincode::deserialize(msg.payload.as_slice()).unwrap();

            match kv_msg.msg_type.as_str() {
                "map" => {
                    let local_map: HashMap<String, String> =
                        bincode::deserialize(kv_msg.payload.as_slice()).unwrap();

                    for (key, value) in local_map.into_iter() {
                        map_.read().unwrap().insert(key, value);
                    }
                }
                "kv_addr" => {
                    let external_node_addr: UuidAddress =
                        bincode::deserialize(kv_msg.payload.as_slice()).unwrap();

                    kv_nodes
                        .read()
                        .unwrap()
                        .insert(external_node_addr.id, external_node_addr.address);
                }
                _ => {}
            }
        }
    });
}

struct MapMemberAddedListener {
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
    kv_nodes: Arc<RwLock<chashmap::CHashMap<Uuid, hover::common::Address>>>,
    kv_address: Arc<RwLock<hover::common::Address>>,
}

impl MapMemberAddedListener {
    fn send_message(&self, msg_type: String, payload: Vec<u8>, node_meta: &NodeMeta) {
        let msg = KvMessage { msg_type, payload };

        let bytes = bincode::serialize(&msg).unwrap();

        self.hover
            .read()
            .unwrap()
            .get_messaging_service()
            .unwrap()
            .read()
            .unwrap()
            .send_to_member(bytes, node_meta);
    }
}

impl EventListener for MapMemberAddedListener {
    fn on_event(&self, event: hover::events::Event) {
        match event {
            hover::events::Event::MemberAdded { node_meta } => {
                let local_map: HashMap<String, String> =
                    HashMap::from_iter(self.map.read().unwrap().clone().into_iter());
                let map_bytes = bincode::serialize(&local_map).unwrap();
                self.send_message(String::from("map"), map_bytes, &node_meta);

                let uuid_address = UuidAddress {
                    id: self.hover.read().unwrap().get_node_id().unwrap().clone(),
                    address: self.kv_address.read().unwrap().clone(),
                };
                let addr_bytes = bincode::serialize(&uuid_address).unwrap();
                self.send_message(String::from("kv_addr"), addr_bytes, &node_meta);
            }
            hover::events::Event::MemberLeft { node_meta } => {
                self.kv_nodes.read().unwrap().remove(&(node_meta.id));
            }
            _ => {}
        }
    }
}

#[derive(Deserialize, Serialize)]
struct KvMessage {
    msg_type: String,
    payload: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
struct UuidAddress {
    id: Uuid,
    address: hover::common::Address,
}
