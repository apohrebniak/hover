extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex, RwLock};

use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use hover::events::EventListener;
use hyper::{Body, Response, StatusCode};
use mime::Mime;
use serde::{Deserialize, Serialize, Serializer};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

pub mod settings;

#[derive(Clone, StateData)]
struct HoverState {
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    value: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct PathStringExtractor {
    key: String,
}

fn get_info(mut state: State) -> (State, Response<Body>) {
    let hover = HoverState::take_from(&mut state).hover;

    let res = create_empty_response(&state, StatusCode::OK);
    (state, res)
}

fn get_members(mut state: State) -> (State, Response<Body>) {
    let hover = HoverState::take_from(&mut state).hover;
    let members = hover
        .read()
        .unwrap()
        .get_cluster_service()
        .map(|ms| ms.read().unwrap().get_members())
        .unwrap();

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_JSON,
        serde_json::to_string(&members).unwrap(),
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

fn post_kv(mut state: State) -> (State, Response<Body>) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let value = QueryStringExtractor::take_from(&mut state).value;
    let hover_state = HoverState::take_from(&mut state);
    let map = hover_state.map;
    let hover = hover_state.hover;

    let inserted_opt = map.read().unwrap().insert(key.clone(), value.clone());

    if let None = inserted_opt {
        let event = MapEvent::Post { key, value };
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
        route.get("/").to(get_info);
        route.get("/members").to(get_members);
        route.get("/kv").to(get_kv_all);
        route
            .get("/kv/:key")
            .with_path_extractor::<PathStringExtractor>()
            .to(get_kv);
        route
            .post("/kv/:key")
            .with_query_string_extractor::<QueryStringExtractor>()
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

    hover.write().unwrap().start();
    setup_hover(hover.clone(), map.clone());

    let hover_state = HoverState { hover, map };
    let router = router(hover_state);

    println!(
        "Listening for requests at http://{}:{}",
        settings.host, settings.port
    );
    gotham::start(
        (
            Ipv4Addr::from_str(settings.host.as_str()).unwrap(),
            settings.port,
        ),
        router,
    )
}

#[derive(Deserialize, Serialize)]
enum MapEvent {
    Post { key: String, value: String },
    Delete { key: String },
}

fn setup_hover(
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
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
    };
    hover
        .read()
        .unwrap()
        .add_event_listener(member_added_listener);

    let map_ = map.clone();
    hover.write().unwrap().add_msg_listener(move |msg| {
        if let hover::common::MessageType::Request = msg.msg_type {
            let local_map: HashMap<String, String> =
                bincode::deserialize(msg.payload.as_slice()).unwrap();

            for (key, value) in local_map.into_iter() {
                map_.read().unwrap().insert(key, value);
            }
        }
    });
}

struct MapMemberAddedListener {
    hover: Arc<RwLock<hover::Hover>>,
    map: Arc<RwLock<chashmap::CHashMap<String, String>>>,
}

impl EventListener for MapMemberAddedListener {
    fn on_event(&self, event: hover::events::Event) {
        if let hover::events::Event::MemberAdded { node_meta } = event {
            let local_map: HashMap<String, String> =
                HashMap::from_iter(self.map.read().unwrap().clone().into_iter());

            let payload = bincode::serialize(&local_map).unwrap();

            self.hover
                .read()
                .unwrap()
                .get_messaging_service()
                .unwrap()
                .read()
                .unwrap()
                .send_to_member(payload, &node_meta);
        }
    }
}
