use crate::{
    game_data::{GraphData, DynamicEdge, StaticEdge, StringID, NodeID},
    randomizer::{Result, Graph},
};
use bimap::BiMap;
use linked_hash_set::LinkedHashSet;
use petgraph::{
    algo,
    graph::{EdgeIndex, NodeIndex},
    stable_graph::{EdgeIndices, StableDiGraph},
    IntoWeightedEdge
};

type Edge = (NodeID, NodeID);

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum SwapEdge {
    OneWay(EdgeIndex),
    TwoWay(EdgeIndex, EdgeIndex),
}

pub struct GameGraph {
    base_graph: StableDiGraph,
    id_map: BiMap<StringID, NodeID>,
    node_map: BiMap<NodeID, NodeIndex>,

    // This needs to be a linked hash set because HashSet iteration order is non-deterministic,
    // which breaks seeded randomization
    swappable_edges: LinkedHashSet<SwapEdge>,
}

// Maps string IDs to u32 IDs for all nodes. Returns the static edges, dynamic edges, 
// and a bidirectional map of string IDs to u32 IDs.
fn assign_numeric_ids(
    str_static_edges: Vec<StaticEdge<StringID>>, 
    str_dynamic_edges: Vec<DynamicEdge<StringID>>
) -> (Vec<Edge>, Vec<Edge>, BiMap<StringID, NodeID>) {
    let unique_id_set: HashSet<String> = 
        str_static_edges.iter()
        .cloned()
        .flat_map(|e| vec![e.start.clone(), e.end.clone()])
        .chain(str_dynamic_edges.iter()
            .cloned()
            .flat_map(|e| vec![e.start.clone(), e.end.clone()]),
        ).collect();

    let set_len: NodeID = unique_id_set.len() as NodeID;
    let id_string_map: BiMap<StringID, NodeID> = unique_id_set.into_iter().zip(0..set_len).collect();

    let static_edges: Vec<StaticEdge<NodeID>> = str_static_edges.iter()
        .map(|e| StaticEdge {
            start: *id_string_map.get_by_left(&e.start)
                .expect("Failed to get Node ID from string when building static edge start"),
            end: *id_string_map.get_by_left(&e.end)
                .expect("Failed to get Node ID from string when building static edge end"),
            two_way: e.two_way,
        }).collect();

    let dynamic_edges: Vec<DynamicEdge<NodeID>> = str_dynamic_edges.iter()
        .map(|e| DynamicEdge {
            start: *id_string_map.get_by_left(&e.start)
                .expect("Failed to get Node ID from string when building dynamic edge start"),
            end: *id_string_map.get_by_left(&e.end)
                .expect("Failed to get Node ID from string when building dynamic edge end"),
            two_way: e.two_way,
        }).collect();

    (static_edges, dynamic_edges, id_string_map)
}

fn build_base_graph(static_edges: Vec<StaticEdge<NodeID>>) -> (StableDiGraph, BiMap<NodeID, NodeIndex>) {
    let mut base_graph_edges = vec![];
    for edge in static_edges {
        if edge.two_way {
            base_graph_edges.push((edge.start, edge.end));
            base_graph_edges.push((edge.end, edge.start));
        } else {
            base_graph_edges.push((edge.start, edge.end));
        }
    }

    let mut graph = StableDiGraph::new();
    let mut node_map = BiMap::new();

    for edge in base_graph_edges {
        let (start_node_id, end_node_id, _) = edge.into_weighted_edge();
        insert_edge(&mut graph, &mut node_map, start_node_id, end_node_id);
    }

    (graph, node_map)
}

// Insert an edge between two existing nodes. If nodes do not exist, create nodes and insert edge.
fn insert_edge(
    graph: &mut StableDiGraph<NodeID, ()>, 
    node_map: &mut BiMap<NodeID, NodeIndex>,
    a: NodeID,
    b: NodeID,
) -> EdgeIndex {
    let (a_idx, b_idx) = match (node_map.get_by_left(&a), node_map.get_by_left(&b)) {
        (Some(a_idx), Some(b_idx)) => (a_idx, b_idx),
        (Some(a_idx), None) => (a_idx, add_node(graph, node_map, b)),
        (None, Some(b_idx)) => (add_node(graph, node_map, a), b_idx),
        (None, None) => (add_node(graph, node_map, a), add_node(graph, node_map, b)),
    };
    graph.add_edge(a_idx, b_idx, ())
}

fn add_node(
    graph: &mut StableDiGraph<NodeID, ()>, 
    node_map: &mut BiMap<NodeID, NodeIndex>,
    node_id: NodeID,
) -> NodeIndex {
    let node_index = graph.add_node(node_id);
    node_map.insert(node_id, node_index);
    node_index
}

fn add_swappable_edges(
    base_graph: &mut StableDiGraph, 
    dynamic_edges: Vec<DynamicEdge<NodeID>>
) -> Vec<SwapEdge> {
    let (two_ways, one_ways): (Vec<DynamicEdge<NodeID>>, Vec<DynamicEdge<NodeID>>) = dynamic_edges
        .into_iter()
        .partition(|e| e.two_way);

    let mut swappable_edges = LinkedHashSet::new();
    for e in one_ways.into_iter() {
        let idx = base_graph.add_edge(e.start, e.end);
        swappable_edges.insert(SwapEdge::OneWay(idx));
    }

    for e in two_ways.into_iter() {
        let idx1 = base_graph.add_edge(e.start, e.end);
        let idx2 = base_graph.add_edge(e.end, e.start);
        swappable_edges.insert(SwapEdge::TwoWay(idx1, idx2));
    }

    swappable_edges
}

impl GameGraph {
    pub fn new(graph_data: GraphData<StringID>) -> Self {
        let (static_edges, dynamic_edges, id_map) = assign_numeric_ids(
            graph_data.static_edges,
            graph_data.dynamic_edges,
        );

        let (mut base_graph, node_map) = build_base_graph(static_edges);
        let swappable_edges = add_swappable_edges(&mut base_graph);

        Self {
            base_graph,
            id_map,
            node_map,
            swappable_edges,
        }
    }

    fn extract_string_ids(&self, idx: EdgeIndex) -> Result<(StringID, StringID)> {
        let (edge_start, edge_end) = self.get_node_ids_for_edge(idx).expect("Failed to get edge endpoints when extracting their string representations");
        let edge_start_string = self.id_map.get_by_right(&edge_start).expect("Failed to extract string representation of start of edge");
        let edge_end_string = self.id_map.get_by_right(&edge_end).expect("Failed to extract string representation of end of edge");
        (edge_start_string.clone(), edge_end_string.clone())
    }

    fn get_node_ids_for_edge(&self, idx: EdgeIndex) -> Result<(NodeID, NodeID)> {
        let (node1, node2) = self.base_graph.edge_endpoints(idx).unwrap();
        let node1_id = self.node_map.get_by_right(&node1)?;
        let node2_id = self.node_map.get_by_right(&node2)?;
        (node1_id, node2_id)
    }

    fn swap_one_ways(&self, idx: EdgeIndex, other_idx: EdgeIndex) -> Result<(SwapEdge, SwapEdge)> {
        let (e1, e2) = self.swap_base_graph_edges(idx, other_idx)?;
        Ok((SwapEdge::OneWay(e1), SwapEdge::OneWay(e2)))
    }

    fn swap_two_ways(
        &self,
        idx1: EdgeIndex,
        idx2: EdgeIndex,
        other_idx1: EdgeIndex,
        other_idx2: EdgeIndex,
    ) -> Result<(SwapEdge, SwapEdge)> {
        let (e1, e2) = self.swap_base_graph_edges(idx1, other_idx1)?;
        let (e3, e4) = self.swap_base_graph_edges(idx2, other_idx2)?;
        Ok((SwapEdge::TwoWay(e1, e4), SwapEdge::TwoWay(e2, e3)))
    }

    fn swap_base_graph_edges(&self, idx1: EdgeIndex, idx2: EdgeIndex) -> Result<(EdgeIndex, EdgeIndex)> {
        let (edge1a, edge1b) = self.get_node_ids_for_edge(idx1)?;
        let (edge2a, edge2b) = self.get_node_ids_for_edge(idx2)?;

        self.base_graph
            .remove_edge(idx1)
            .unwrap_or_else(|| panic!("Failed to remove edge ({:?}, {:?})", edge1a, edge1b));
        self.base_graph
            .remove_edge(idx2)
            .unwrap_or_else(|| panic!("Failed to remove edge ({:?}, {:?})", edge2a, edge2b));

        let new_edge_idx1 = self.base_graph.add_edge(edge1a, edge2b);
        let new_edge_idx2 = self.base_graph.add_edge(edge2a, edge1b);

        (new_edge_idx1, new_edge_idx2)
    }
}


impl Graph<NodeID, SwapEdge> for GameGraph {
    // TODO: There's an extra layer of transformations here. We should use numeric IDs to begin
    // with instead of string IDs and just add string descriptions.
    fn get_edges(&self) -> Result<Vec<(StringID, StringID)>> {
        let mut res: Vec<(String, String)> = vec![];
        for edge in self.swappable_edges {
            match edge {
                SwapEdge::OneWay(idx) => res.push(self.extract_string_ids(idx)?),
                SwapEdge::TwoWay(idx1, idx2) => {
                    res.push(self.extract_string_ids(idx1)?);
                    res.push(self.extract_string_ids(idx2)?);
                },
            }
        }

        for (start, end) in &res {
            println!("id pair: {}, {}", start, end);
        }

        Ok(res)
    }

    fn swap_edges(&mut self, edge1: SwapEdge, edge2: SwapEdge) -> Result<(SwapEdge, SwapEdge)> {
        use SwapEdge;
        let (new_edge1, new_edge2) = match (edge1, edge2) {
            (OneWay(idx), OneWay(other_idx)) => self.swap_one_ways(idx1, idx2),
            (TwoWay(idx1, idx2), TwoWay(other_idx1, other_idx4)) => self.swap_two_ways(idx1, idx2, other_idx1, other_idx2),
            _ => Err(),
        }?;

        if !self.swappable_edges.remove(&edge1) {
            return Err(format!(
                "Failed to remove {:?} from swappable edges",
                &edge1
            ))
        }

        if !self.swappable_edges.remove(&edge2) {
            return Err(format!(
                "Failed to remove {:?} from swappable edges",
                &edge2
            ))
        }

        self.swappable_edges.insert(new_edge1);
        self.swappable_edges.insert(new_edge2);

        Ok((new_edge1, new_edge2))
    }

    fn get_unreachable_regions(&self) -> Vec<Vec<NodeID>> {
        let condensed_graph = algo::condensation(
            self.base_graph.map(|_, n| n, |_, e| e).into(),
            /* make_acyclic */ true,
        );

        // A bit slow, but the remote possibility of panicking and killing the 
        // entire server scared me away from the faster solution
        condensed_graph
            .node_indices()
            .zip(condensed_graph.node_weights())
            .filter(|(idx, node_ids)| condensed_graph
                .externals(Direction::Incoming).find(idx).is_some())
            .map(|(_idx, node_ids)| node_ids)
            .collect()
    }
}
