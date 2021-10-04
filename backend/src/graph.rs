use crate::randomizer::Graph;
use petgraph;

pub struct GameGraph;

impl Graph<u32, u32> for GameGraph {
    fn edge_count(&self) -> usize {
        0
    }
    fn edge_endpoints(&self, e: u32) -> Option<(u32, u32)> {
        None
    }
    fn edge_indices(&self) -> Vec<u32> {
        vec![]
    }
    fn add_edge(&mut self, node1: u32, node2: u32) -> u32 {
        0
    }
    fn remove_edge(&mut self, e: u32) -> Option<()> {
        None
    }
}
