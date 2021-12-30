use crate::randomizer::{Graph, Result};

pub fn is_beatable<N, E>(graph: &impl Graph<N, E>) -> bool {
    graph.get_unreachable_regions().len() == 1
}


pub fn standard_shuffle<N, E>(graph: &mut impl Graph<N, E>, rng: &mut impl Rng) -> Result<()> {
    // TODO: this assumption should already be checked upon loading the graph data
    if !is_beatable(graph) {
        return Err(Error::GameUnbeatable("Initial graph unbeatable"));
    }

    // TODO: Make this configurable
    let iterations = 100;

    for _ in 0..iterations {
        if let Some((edge1, edge2)) = graph.pick_random_edges(rng) {
            let (new_edge1, new_edge2) = graph.swap_edges(edge1, edge2)?;

            if !is_beatable(&graph) {
                graph.swap_edges(new_edge1, new_edge2)?;
            }
        }
    }

    Ok(())
}

pub fn chaos_shuffle<N, E>(graph: &mut impl Graph<N, E>) -> Result<()> {
    Ok(())
}
