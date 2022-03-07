use crate::{error::Result, graph::Graph};
use std::{collections::HashMap, fmt::Debug};

type Address = usize;
type StringID = String;
type Destination = [u8; 4];

// maps for converting randomized game data back into ROM addresses
#[derive(Clone)]
pub struct RomDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}

pub trait Rom {
    fn write_data<N: Debug, E>(
        &mut self,
        rom_data_maps: &RomDataMaps,
        graph: &mut impl Graph<N, E>,
    ) -> Result<()>;
}
