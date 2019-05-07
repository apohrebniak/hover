extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;

use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;
use std::sync::{Arc, Mutex, RwLock};

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

fn get_info(mut state: State) -> (State, String) {
    let hover = HoverState::take_from(&mut state).hover;

    (state, String::from("Info"))
}

fn get_members(mut state: State) -> (State, String) {
    let hover = HoverState::take_from(&mut state).hover;

    (state, String::from("Members"))
}

fn get_kv_all(mut state: State) -> (State, String) {
    let map = HoverState::take_from(&mut state).map;

    (state, String::from("KV all"))
}

fn get_kv(mut state: State) -> (State, String) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let map = HoverState::take_from(&mut state).map;

    (
        state,
        format!("{:?}", format_args!("Requested parameter {:?}", key)),
    )
}

fn post_kv(mut state: State) -> (State, String) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let map = HoverState::take_from(&mut state).map;

    (
        state,
        format!("{:?}", format_args!("Post parameter {:?}", key)),
    )
}

fn delete_kv(mut state: State) -> (State, String) {
    let key = PathStringExtractor::take_from(&mut state).key;
    let map = HoverState::take_from(&mut state).map;

    (
        state,
        format!("{:?}", format_args!("Delete parameter {:?}", key)),
    )
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
    let addr = "127.0.0.1:9090";
    println!("Starting hover...");

    let hover = hover::Hover::default()
        .map(|h| Arc::new(RwLock::new(h)))
        .unwrap();

    hover.write().unwrap().start();

    let hover_ = hover.clone();
    hover
        .write()
        .unwrap()
        .add_broadcast_listener(|msg| {
            let event: MapEvent = bincode::deserialize(msg.payload.as_slice()).unwrap();

            match event {
                MapEvent::Post { key, value } => {}
                MapEvent::Delete { key } => {}
            }
            x
        })
        .unwrap();

    let hover_state = HoverState {
        hover: hover.clone(),
        map: Arc::new(RwLock::new(chashmap::CHashMap::new())),
    };

    let router = router(hover_state);

    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router)
}

#[derive(Deserialize, Serialize)]
enum MapEvent {
    Post { key: String, value: String },
    Delete { key: String },
}