use crate::rng::{ChooseMultipleFill, RandomBool};
use std::fmt::Debug;
use thiserror::Error;

type NodeID = u32;

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

#[derive(Error, Debug)]
pub enum GetStringIDsError {
    #[error("Error getting string IDs: {0}")]
    EdgeEndpoints(#[from] GetEdgeEndpointsError),
    #[error("Could not get string ID for node with ID {0} of edge with index {1}")]
    MissingStringID(NodeID, usize),
}

pub trait Graph<N, E> {
    fn swap_edges(&mut self, edge1: E, edge2: E) -> std::result::Result<(E, E), EdgeSwapError>;
    fn pick_random_edges<R>(&self, rng: &mut R) -> Option<(E, E)>
    where
        R: RandomBool + ChooseMultipleFill;
    // TODO: Change this to return Vec<(N, N)> once string IDs are moved to descriptions
    fn get_edges(&self) -> Vec<(String, String)>;
    fn get_unreachable_regions(&self) -> Vec<Vec<N>>
    where
        N: Debug;
}
