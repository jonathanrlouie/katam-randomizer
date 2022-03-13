use crate::rng::{ChooseMultipleFill, RandomBool};
use std::{
    cmp::Eq,
    hash::Hash,
    collections::HashMap,
    fmt::Debug
};
use thiserror::Error;

type NodeID = String;
type Address = usize;
type Destination = [u8; 4];

#[derive(Copy, Clone, Debug)]
pub enum SwapEdgeIndices {
    OneWay(usize),
    TwoWay(usize, usize),
}

#[derive(Error, Debug)]
pub enum GetEdgeEndpointsError {
    #[error("No endpoints found for edge with index {0}")]
    NoEndpoints(usize),
    #[error(
        "No node ID found for node with index {0} when getting endpoints for edge with index {1}"
    )]
    MissingNodeID(usize, usize),
}

#[derive(Error, Debug)]
pub enum BaseEdgeSwapError {
    #[error("Failed to remove edge ({0}, {1}) with index {2}")]
    MissingBaseEdge(NodeID, NodeID, usize),
    #[error("Error swapping base edges: {0}")]
    EdgeEndpoints(#[from] GetEdgeEndpointsError),
}

#[derive(Error, Debug)]
pub enum EdgeSwapError {
    #[error("Failed to swap edge {0:?} with edge {1:?} of opposite type")]
    Mismatch(SwapEdgeIndices, SwapEdgeIndices),
    #[error("Edge {0:?} is not a swappable edge of the graph")]
    NonSwappableEdge(SwapEdgeIndices),
    #[error("Error swapping edges: {0}")]
    BaseEdgeSwap(#[from] BaseEdgeSwapError),
}

pub trait Graph<N, E> {
    fn swap_edges(&mut self, edge1: E, edge2: E) -> Result<(E, E), EdgeSwapError>;
    fn pick_random_edges<R>(&self, rng: &mut R) -> Option<(E, E)>
    where
        R: RandomBool + ChooseMultipleFill;
    fn get_edges(&self) -> Vec<(N, N)>;
    fn get_unreachable_regions(&self) -> Vec<Vec<N>>
    where
        N: Debug;
}

pub trait DoorData<N: Eq + Hash> {
    fn door_data(&self) -> HashMap<N, (Destination, Vec<Address>)>;
}
