use std::collections::HashMap;

pub type Address = usize;
pub type StringID = String;
pub type Destination = [u8; 4];

pub type NodeID = u32;

// maps for converting randomized game data back into ROM addresses
#[derive(Clone)]
pub struct RomDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}