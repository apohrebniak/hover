extern crate bincode;
extern crate serde;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};

pub fn to_bytes<T: ?Sized>(val: &T) -> Result<Vec<u8>, Box<Error>>
where
    T: serde::Serialize,
{
    match bincode::serialize(val) {
        Ok(vector) => Ok(vector),
        Err(ex) => Err(Box::new(ex)),
    }
}

pub fn from_bytes<'a, T: ?Sized>(bytes: &'a [u8]) -> Result<T, Box<Error>>
where
    T: serde::de::Deserialize<'a>,
{
    match bincode::deserialize(bytes) {
        Ok(obj) => Ok(obj),
        Err(ex) => Err(Box::new(ex)),
    }
}
