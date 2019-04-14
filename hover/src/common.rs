extern crate serde_json;
extern crate serde_repr;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::net::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Eq)]
pub struct Address {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum MessageType {
    REQUEST = 0,
    RESPONSE = 1,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct Message {
    pub correlation: Option<u32>,
    pub r#type: MessageType,
    pub payload: Vec<u8>,
    pub return_address: Option<Address>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Hash)]
#[repr(u8)]
enum ConnectionMessageType {
    Try = 0,
    Ok = 1,
    Duplicate = 2,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
struct ConnectionMessage {
    r#type: ConnectionMessageType,
    node_id: String,
}
