use crate::{
    config::{self, EntranceShuffleType},
};
use thiserror::Error;

pub trait RNG {
    fn get_bool(&mut self, p: f64) -> bool;
}

pub trait RomWriter {
    fn write_data<N, E>(&mut self, data: impl Graph<N, E>) -> anyhow::Result<()>;
}

pub trait RandoAlgorithms<N, E> {
    fn standard_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()>;
    fn chaos_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()>;
}

pub trait Graph<NodeID, EdgeIndex> {
    fn edge_count(&self) -> usize;
    fn edge_endpoints(&self, e: EdgeIndex) -> Option<(NodeID, NodeID)>;
    fn edge_indices(&self) -> Vec<EdgeIndex>;
    fn add_edge(&mut self, node1: NodeID, node2: NodeID) -> EdgeIndex;
    fn remove_edge(&mut self, e: EdgeIndex) -> Option<()>;
}

pub fn randomize_katam<N, E, G: Graph<N, E>>(
    config: config::Config, 
    mut rng: impl RNG,
    mut rom_writer: impl RomWriter,
    algorithms: impl RandoAlgorithms<N, E>,
    mut graph: G,
    ) -> anyhow::Result<()> {
    match config.entrance_shuffle {
        EntranceShuffleType::Standard => algorithms.standard_shuffle(&mut graph),
        EntranceShuffleType::Chaos => algorithms.chaos_shuffle(&mut graph)
    }?;
    rom_writer.write_data(graph)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockConfig;

    const MOCK_CONFIG: MockConfig = MockConfig;
    struct MockRng;

    impl RNG for MockRng {
        fn get_bool(&mut self, p: f64) -> bool {
            true
        }
    }

    struct MockAlgorithms;
    impl<N, E> RandoAlgorithms<N, E> for MockAlgorithms {
        fn standard_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()> {
            Ok(())
        }
        fn chaos_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    impl RNG for MockAlgorithms {
        fn get_bool(&mut self, p: f64) -> bool {
            true
        }
    }

    struct MockRomWriter;

    impl RomWriter for MockRomWriter {
        fn write_data<N, E>(&mut self, data: impl Graph<N, E>) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct MockGraph;

    impl Graph<u32, u32> for MockGraph {
        fn edge_count(&self) -> usize { 0 }
        fn edge_endpoints(&self, e: u32) -> Option<(u32, u32)> { None }
        fn edge_indices(&self) -> Vec<u32> { vec![] }
        fn add_edge(&mut self, node1: u32, node2: u32) -> u32 { 0 }
        fn remove_edge(&mut self, e: u32) -> Option<()> { None }
    }
    
    #[test]
    fn test_randomize_game() -> anyhow::Result<()> {
        randomize_katam(MOCK_CONFIG, MockRng, MockRomWriter, MockAlgorithms, MockGraph)
    }
}
