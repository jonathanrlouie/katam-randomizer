use crate::{
    algorithm,
    config::{self, EntranceShuffleType}
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KatamRandoError {
    #[error("Randomized game cannot be beaten.")]
    Unbeatable,
    #[error(transparent)]
    RomWriteError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type Result<T> = std::result::Result<T, KatamRandoError>;

pub trait RNG {
    fn get_bool(&mut self, p: f64) -> bool;
}

pub trait RomWriter {
    fn write_data<N, E>(&mut self, data: impl Graph<N, E>) -> Result<()>;
}

pub trait Graph<N, E> {
    fn swap_edges(&mut self, edge1: E, edge2: E) -> Result<(E, E)>;
    fn pick_random_edges(&self, rng: &mut impl Rng) -> Option<(E, E)>;
    // TODO: Change this to return Result<Vec<(N, N)>> once string IDs are moved to descriptions
    fn get_edges(&self) -> Result<Vec<(String, String)>>;
    fn get_unreachable_regions(&self) -> Vec<Vec<N>>;
}

pub fn randomize_katam<N, E, G: Graph<N, E>>(
    config: config::Config,
    mut rng: impl RNG,
    mut rom_writer: impl RomWriter,
    mut graph: G,
) -> Result<()> {
    match config.entrance_shuffle {
        EntranceShuffleType::Standard => algorithm::standard_shuffle(&mut graph, &mut rng),
        EntranceShuffleType::Chaos => algorithm::chaos_shuffle(&mut graph, &mut rng),
    }?;
    rom_writer.write_data(graph)?;
    Ok(())
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

    impl RNG for MockRng {
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
