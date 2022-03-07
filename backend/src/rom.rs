use crate::graph::Graph;
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

type Address = usize;
type StringID = String;
type Destination = [u8; 4];

#[derive(Error, Debug)]
#[error("Error writing byte {byte:#04x} at address {address}")]
pub struct ByteWriteError {
    pub byte: u8,
    pub address: Address,
}

#[derive(Error, Debug)]
#[error("Errors writing bytes to addresses: {0:?}")]
pub struct WriteAddressesError(pub Vec<ByteWriteError>);

// maps for converting randomized game data back into ROM addresses
#[derive(Clone)]
pub struct RomDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}

pub trait Rom {
    fn write_data<N, E>(
        &mut self,
        rom_data_maps: &RomDataMaps,
        graph: &mut impl Graph<N, E>,
    ) -> Result<(), std::io::Error>;
}
