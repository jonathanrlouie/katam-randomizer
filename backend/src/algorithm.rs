use crate::randomizer::{Graph, Result};

pub fn is_beatable<N, E>(graph: &impl Graph<N, E>) -> bool {
    let condensed_graph = condensation(
        self.base_graph.map(|_, n| n, |_, e| e).into(),
        /*make_acyclic*/ true,
    );

    if condensed_graph.externals(Direction::Incoming).count() == 1 {
        Ok(())
    } else {
        let node_ids = condensed_graph
            .externals(Direction::Incoming)
            .flat_map(|idx| condensed_graph.node_weight(idx).unwrap())
            .map(|id| **id)
            .collect::<Vec<NodeID>>();
        Err(Error::GameUnbeatable(node_ids))
    }
}

pub fn standard_shuffle<N, E>(graph: &mut impl Graph<N, E>, rng: &mut impl Rng) -> Result<()> {
    // TODO: this assumption should already be checked upon loading the graph data
    if !is_beatable(graph) {
        return Err(Error::GameUnbeatable("Initial graph unbeatable"));
    }

    for _ in 0..iterations {
        if rng.get_bool(0.5f64) {
            // one ways case
            if let Some((edge1, edge2)) = pick_random_edges(swap_edges, rng) {
                let (new_edge1, new_edge2) = graph.swap_edges(edge1, edge2)?;

                if !is_beatable(&graph) {
                    graph.swap_edges(new_edge1, new_edge2);
                }
            }
        } else {
            // two ways case
            if let Some((edge1, edge2)) = pick_random_edges(swap_edges, rng) {
                let (new_edge1, new_edge2) = graph.swap_edges(edge1, edge2)?;

                if !is_beatable(&graph) {
                    graph.swap_edges(new_edge1, new_edge2);
                }
            }
        }
    }

    Ok(())
}

pub fn chaos_shuffle<N, E>(graph: &mut impl Graph<N, E>) -> Result<()> {
    Ok(())
}
