use std::fmt::Debug;
use crate::{
    config::{self, EntranceShuffleType},
    error::{EdgeSwapError, Result},
    types::RomDataMaps,
};

pub trait Rng {
    fn get_bool(&mut self, p: f64) -> bool;
    fn choose_multiple_fill<T, I: Iterator<Item=T>>(&mut self, iter: I, buf: &mut [T]) -> usize;
}

pub trait Rom {
    fn write_data<N: Debug, E>(&mut self, rom_data_maps: RomDataMaps, graph: impl Graph<N, E>) -> Result<()>;
}

pub trait Graph<N: Debug, E> {
    fn swap_edges(&mut self, edge1: E, edge2: E) -> std::result::Result<(E, E), EdgeSwapError>;
    fn pick_random_edges(&self, rng: &mut impl Rng) -> Option<(E, E)>;
    // TODO: Change this to return Vec<(N, N)> once string IDs are moved to descriptions
    fn get_edges(&self) -> Vec<(String, String)>;
    fn get_unreachable_regions(&self) -> Vec<Vec<N>>;
}

pub fn randomize_katam<N: Debug, E, G: Graph<N, E>>(
    config: config::Config,
    mut rng: impl Rng,
    mut rom: impl Rom,
    rom_data_maps: RomDataMaps,
    mut graph: G,
) -> Result<()> {
    match config.entrance_shuffle {
        EntranceShuffleType::Standard => standard_shuffle(&mut graph, &mut rng),
        EntranceShuffleType::Chaos => chaos_shuffle(&mut graph, &mut rng),
    };
    rom.write_data(rom_data_maps, graph)?;
    Ok(())
}

pub fn is_beatable<N: Debug, E>(graph: &impl Graph<N, E>) -> bool {
    graph.get_unreachable_regions().len() == 1
}

fn standard_shuffle<N: Debug, E>(graph: &mut impl Graph<N, E>, rng: &mut impl Rng) -> () {
    // TODO: this assumption should already be checked upon loading the graph data
    if !is_beatable(graph) {
        panic!("Initial graph unbeatable. Unreachable regions: {:?}", graph.get_unreachable_regions());
    }

    // TODO: Make this configurable
    let iterations = 100;

    for _ in 0..iterations {
        if let Some((edge1, edge2)) = graph.pick_random_edges(rng) {
            let (new_edge1, new_edge2) = graph.swap_edges(edge1, edge2).expect("Standard shuffle: Swapping edges failed");

            if !is_beatable(graph) {
                graph.swap_edges(new_edge1, new_edge2).expect("Standard shuffle: Swapping back edges failed");
            }
        }
    }
}

fn chaos_shuffle<N: Debug, E>(graph: &mut impl Graph<N, E>, rng: &mut impl Rng) -> () {
    ()
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;

    const MOCK_CONFIG: Config = config::Config {
        seed: 0,
        entrance_shuffle: EntranceShuffleType::Standard,
    };

    struct MockRng;

    impl Rng for MockRng {
        fn get_bool(&mut self, _p: f64) -> bool {
            true
        }
    }

    struct MockRomWriter;

    impl RomWriter for MockRomWriter {
        fn write_data<N, E>(&mut self, _data: impl Graph<N, E>) -> Result<()> {
            Ok(())
        }
    }

    struct MockGraph;

    impl Graph<u32, u32> for MockGraph {
        fn edge_count(&self) -> usize {
            0
        }
        fn edge_endpoints(&self, _e: u32) -> Option<(u32, u32)> {
            None
        }
        fn edge_indices(&self) -> Vec<u32> {
            vec![]
        }
        fn add_edge(&mut self, _node1: u32, _node2: u32) -> u32 {
            0
        }
        fn remove_edge(&mut self, _e: u32) -> Option<()> {
            None
        }
    }

    #[test]
    fn test_randomize_game() -> Result<()> {
        randomize_katam(
            MOCK_CONFIG,
            MockRng,
            MockRomWriter,
            MockAlgorithms,
            MockGraph,
        )
    }
}
