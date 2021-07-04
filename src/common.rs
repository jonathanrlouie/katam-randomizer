use std::collections::HashMap;

pub type StringID = String;
type Address = usize;
type Destination = [u8; 4];

pub struct DoorDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}
