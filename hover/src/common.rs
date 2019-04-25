use std::net::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Eq, Clone)]
pub struct NodeMeta {
    pub id: Uuid,
    pub addr: Address,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Eq, Clone)]
pub struct Address {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub enum MessageType {
    REQUEST = 0,
    RESPONSE = 1,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct Message {
    pub corId: Uuid,
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
    pub return_address: Option<Address>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
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
