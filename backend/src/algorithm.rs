use crate::randomizer::{Graph, RandoAlgorithms};

pub struct KatamAlgorithms;

impl<N, E> RandoAlgorithms<N, E> for KatamAlgorithms {
    fn standard_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()> {
        Ok(())
    }
    fn chaos_shuffle(&self, graph: &mut impl Graph<N, E>) -> anyhow::Result<()> {
        Ok(())
    }
}
