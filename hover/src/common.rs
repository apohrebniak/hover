use std::net::*;

#[derive(PartialEq, Eq, Hash)]
pub struct Address {
    pub ip: Ipv4Addr,
    pub port: u16,
}

//TODO: message placeholder
pub struct Message {}
