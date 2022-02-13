use thiserror::Error;
use crate::types::{Address, NodeID};

#[derive(Error, Debug)]
pub enum GetStringIDsError {
    #[error("Error getting string IDs: {0}")]
    EdgeEndpoints(#[from] GetEdgeEndpointsError),
    #[error("Could not get string ID for node with ID {0} of edge with index {1}")]
    MissingStringID(NodeID, usize),
}

#[derive(Error, Debug)]
pub enum GetEdgeEndpointsError {
    #[error("No endpoints found for edge with index {0}")]
    NoEndpoints(usize),
    #[error("No node ID found for node with index {0} when getting endpoints for edge with index {1}")]
    MissingNodeID(usize, usize),
}

#[derive(Copy, Clone, Debug)]
pub enum SwapEdgeIndices {
    OneWay(usize),
    TwoWay(usize, usize),
}

#[derive(Error, Debug)]
pub enum EdgeSwapError {
    #[error("Failed to swap edge {0} with edge {1} of opposite type")]
    Mismatch(SwapEdgeIndices, SwapEdgeIndices),
    #[error("Edge {0} is not a swappable edge of the graph")]
    NonSwappableEdge(SwapEdgeIndices),
    #[error("Error swapping edges: {0}")]
    BaseEdgeSwap(#[from] BaseEdgeSwapError),
}

#[derive(Error, Debug)]
pub enum BaseEdgeSwapError {
    #[error("Failed to remove edge ({0}, {1}) with index {2}")]
    MissingBaseEdge(NodeID, NodeID, usize),
    #[error("Error swapping base edges: {0}")]
    EdgeEndpoints(#[from] GetEdgeEndpointsError),
}

#[derive(Error, Debug)]
#[error("Error writing byte {byte:#04x} at address {address}")]
pub struct ByteWriteError {
    pub byte: u8,
    pub address: Address,
}

#[derive(Error, Debug)]
pub struct WriteAddressesError(pub Vec<ByteWriteError>);

impl std::fmt::Display for WriteAddressesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined: String = self.0
            .iter()
            .map(|err| err.to_string())
            .join("\n");
        write!(f, "{}", joined)
    }
}

#[derive(Error, Debug)]
pub enum RomDataMapError {
    #[error("No door data found for string ID {0}")]
    MissingDoorData(String)
}

#[derive(Error, Debug)]
pub enum KatamRandoError {
    RomIO(#[from] std::io::Error)
}

pub type Result<T> = std::result::Result<T, KatamRandoError>;
