use crate::graph::{DoorData, Graph};
use std::{cmp::Eq, fmt::Debug, hash::Hash};
use thiserror::Error;

type Address = usize;

#[derive(Error, Debug)]
#[error("Error writing byte {byte:#04x} at address {address}")]
pub struct ByteWriteError {
    pub byte: u8,
    pub address: Address,
}

#[derive(Error, Debug)]
#[error("Errors writing bytes to addresses: {0:?}")]
pub struct WriteAddressesError(pub Vec<ByteWriteError>);

pub trait Rom {
    fn write_data<N, E, G>(&mut self, graph: &mut G) -> Result<(), std::io::Error>
    where
        N: Debug + Eq + Hash,
        G: Graph<N, E> + DoorData<N>;
}
