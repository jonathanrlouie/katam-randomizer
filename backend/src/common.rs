use std::{collections::HashMap, fmt};
use thiserror::Error;

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}


pub type StringID = String;
pub type Address = usize;
pub type Destination = [u8; 4];

pub struct WriteData {
    pub bytes: Vec<u8>,
    pub target_addresses: Vec<usize>,
}

pub struct DoorDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}
