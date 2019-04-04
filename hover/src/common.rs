extern crate serde_json;
extern crate serde_repr;

use std::net::*;
use serde_repr::{Serialize_repr, Deserialize_repr};
use serde::{Deserialize, Serialize};

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
