use crate::error;
use petgraph;

pub struct GameGraph;

struct Beatable;

type EdgeIndex = usize;
type NodeID = usize;

pub trait Graph {
    fn from_edges(edges: &[(NodeID, NodeID)]) -> Self;
    fn edge_count(&self) -> usize;
    fn edge_endpoints(&self, e: EdgeIndex) -> Option<(NodeID, NodeID)>;
    fn edge_indices(&self) -> Vec<EdgeIndex>;
    fn add_edge(&mut self, node1: NodeID, node2: NodeID) -> EdgeIndex;
    fn remove_edge(&mut self, e: EdgeIndex) -> Option<()>;
    fn game_beatable(&self) -> Result<Beatable, error::Error>;
}
