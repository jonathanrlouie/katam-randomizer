use std::collections::{HashSet, HashMap};
use crate::{
    error::{GetStringIDsError, GetEdgeEndpointsError, EdgeSwapError, BaseEdgeSwapError, SwapEdgeIndices},
    randomizer::{Graph, Rng},
    types::{Address, StringID, Destination, NodeID},
};
use bimap::BiMap;
use linked_hash_set::LinkedHashSet;
use serde::{Deserialize, Serialize};
use petgraph::{
    algo,
    Direction,
    graph::{EdgeIndex, NodeIndex},
    stable_graph::StableDiGraph,
    IntoWeightedEdge
};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct StaticEdge<IDType> {
    pub start: IDType,
    pub end: IDType,
    pub two_way: bool,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct DynamicEdge<IDType> {
    pub start: IDType,
    pub end: IDType,
    pub two_way: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GraphData<IDType> {
    pub door_data: HashMap<StringID, (Destination, Vec<Address>)>,
    pub static_edges: Vec<StaticEdge<IDType>>,
    pub dynamic_edges: Vec<DynamicEdge<IDType>>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum SwapEdge {
    OneWay(EdgeIndex),
    TwoWay(EdgeIndex, EdgeIndex),
}

impl From<SwapEdge> for SwapEdgeIndices {
    fn from(edge: SwapEdge) -> Self {
        match edge {
            SwapEdge::OneWay(idx) => SwapEdgeIndices::OneWay(idx.index()),
            SwapEdge::TwoWay(idx1, idx2) => SwapEdgeIndices::TwoWay(idx1.index(), idx2.index()),
        }
    }
}

#[derive(Clone)]
pub struct GameGraph {
    base_graph: StableDiGraph<NodeID, ()>,
    id_map: BiMap<StringID, NodeID>,
    node_map: BiMap<NodeID, NodeIndex>,

    // This needs to be a linked hash set because HashSet iteration order is non-deterministic,
    // which breaks seeded randomization
    swappable_edges: LinkedHashSet<SwapEdge>,
}

// TODO: Remove this function once string IDs are removed
// Maps string IDs to u32 IDs for all nodes. Returns the static edges, dynamic edges, 
// and a bidirectional map of string IDs to u32 IDs.
fn assign_numeric_ids(
    str_static_edges: Vec<StaticEdge<StringID>>, 
    str_dynamic_edges: Vec<DynamicEdge<StringID>>
) -> (Vec<StaticEdge<NodeID>>, Vec<DynamicEdge<NodeID>>, BiMap<StringID, NodeID>) {
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

fn build_base_graph(static_edges: Vec<StaticEdge<NodeID>>) -> (StableDiGraph<NodeID, ()>, BiMap<NodeID, NodeIndex>) {
    let mut base_graph_edges: Vec<(NodeID, NodeID)> = vec![];
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
        let (start_node_id, end_node_id, _): (NodeID, NodeID, (NodeID, NodeID)) = edge.into_weighted_edge();
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
        (Some(a_idx), Some(b_idx)) => (*a_idx, *b_idx),
        (Some(a_idx), None) => (*a_idx, add_node(graph, node_map, b)),
        (None, Some(b_idx)) => (add_node(graph, node_map, a), *b_idx),
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
    base_graph: &mut StableDiGraph<NodeID, ()>, 
    node_map: &mut BiMap<NodeID, NodeIndex>,
    dynamic_edges: Vec<DynamicEdge<NodeID>>
) -> LinkedHashSet<SwapEdge> {
    let (two_ways, one_ways): (Vec<DynamicEdge<NodeID>>, Vec<DynamicEdge<NodeID>>) = dynamic_edges
        .into_iter()
        .partition(|e| e.two_way);

    let mut swappable_edges = LinkedHashSet::new();
    for e in one_ways.into_iter() {
        let idx = insert_edge(base_graph, node_map, e.start, e.end);
        swappable_edges.insert(SwapEdge::OneWay(idx));
    }

    for e in two_ways.into_iter() {
        let idx1 = insert_edge(base_graph, node_map, e.start, e.end);
        let idx2 = insert_edge(base_graph, node_map, e.end, e.start);
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
        let swappable_edges = add_swappable_edges(&mut base_graph, &mut node_map, dynamic_edges);

        Self {
            base_graph,
            id_map,
            node_map,
            swappable_edges,
        }
    }

    fn extract_string_ids(&self, idx: EdgeIndex) -> std::result::Result<(StringID, StringID), GetStringIDsError> {
        let (edge_start, edge_end) = self.get_node_ids_for_edge(idx).map_err(|e| GetStringIDsError::EdgeEndpoints(e))?;
        let edge_start_string = self.id_map.get_by_right(&edge_start).ok_or_else(|| GetStringIDsError::MissingStringID(edge_start, idx.index()))?;
        let edge_end_string = self.id_map.get_by_right(&edge_end).ok_or_else(|| GetStringIDsError::MissingStringID(edge_end, idx.index()))?;
        Ok((edge_start_string.clone(), edge_end_string.clone()))
    }

    fn get_node_ids_for_edge(&self, idx: EdgeIndex) -> std::result::Result<(NodeID, NodeID), GetEdgeEndpointsError> {
        let (node1, node2) = self.base_graph.edge_endpoints(idx).ok_or_else(|| GetEdgeEndpointsError::NoEndpoints(idx.index()))?;
        let node1_id = self.node_map.get_by_right(&node1).ok_or_else(|| GetEdgeEndpointsError::MissingNodeID(idx.index(), node1.index()))?;
        let node2_id = self.node_map.get_by_right(&node2).ok_or_else(|| GetEdgeEndpointsError::MissingNodeID(idx.index(), node2.index()))?;
        Ok((*node1_id, *node2_id))
    }

    fn swap_one_ways(&self, idx: EdgeIndex, other_idx: EdgeIndex) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        let (e1, e2) = self.swap_base_graph_edges(idx, other_idx).map_err(|e| EdgeSwapError::BaseEdgeSwap(e))?;
        Ok((SwapEdge::OneWay(e1), SwapEdge::OneWay(e2)))
    }

    fn swap_two_ways(
        &self,
        idx1: EdgeIndex,
        idx2: EdgeIndex,
        other_idx1: EdgeIndex,
        other_idx2: EdgeIndex,
    ) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        let (e1, e2) = self.swap_base_graph_edges(idx1, other_idx1).map_err(|e| EdgeSwapError::BaseEdgeSwap(e))?;
        let (e3, e4) = self.swap_base_graph_edges(idx2, other_idx2).map_err(|e| EdgeSwapError::BaseEdgeSwap(e))?;
        Ok((SwapEdge::TwoWay(e1, e4), SwapEdge::TwoWay(e2, e3)))
    }

    fn swap_base_graph_edges(&self, idx1: EdgeIndex, idx2: EdgeIndex) -> std::result::Result<(EdgeIndex, EdgeIndex), BaseEdgeSwapError> {
        let (edge1a, edge1b) = self.get_node_ids_for_edge(idx1).map_err(|e| BaseEdgeSwapError::EdgeEndpoints(e))?;
        let (edge2a, edge2b) = self.get_node_ids_for_edge(idx2).map_err(|e| BaseEdgeSwapError::EdgeEndpoints(e))?;

        self.base_graph
            .remove_edge(idx1)
            .ok_or_else(|| BaseEdgeSwapError::MissingBaseEdge(edge1a, edge1b, idx1.index()))?;
        self.base_graph
            .remove_edge(idx2)
            .ok_or_else(|| BaseEdgeSwapError::MissingBaseEdge(edge2a, edge2b, idx2.index()))?;

        let new_edge_idx1 = insert_edge(&mut self.base_graph, &mut self.node_map, edge1a, edge2b);
        let new_edge_idx2 = insert_edge(&mut self.base_graph, &mut self.node_map, edge2a, edge1b);

        Ok((new_edge_idx1, new_edge_idx2))
    }
}


impl Graph<NodeID, SwapEdge> for GameGraph {
    // TODO: There's an extra layer of transformations here. We should use numeric IDs to begin
    // with instead of string IDs and just add string descriptions.
    fn get_edges(&self) -> Vec<(StringID, StringID)> {
        let mut res: Vec<(String, String)> = vec![];
        for edge in self.swappable_edges {
            match edge {
                SwapEdge::OneWay(idx) => res.push(self.extract_string_ids(idx).unwrap_or_else(|e| panic!("Error extracting string IDs for one way edge: {}", e))),
                SwapEdge::TwoWay(idx1, idx2) => {
                    res.push(self.extract_string_ids(idx1).unwrap_or_else(|e| panic!("Error extracting string IDs for two way edge (first): {}", e)));
                    res.push(self.extract_string_ids(idx2).unwrap_or_else(|e| panic!("Error extracting string IDs for two way edge (second): {}", e)));
                },
            }
        }

        for (start, end) in &res {
            // TODO: Debug log level
            println!("id pair: {}, {}", start, end);
        }

        res
    }

    fn swap_edges(&mut self, edge1: SwapEdge, edge2: SwapEdge) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        use SwapEdge::*;
        let (new_edge1, new_edge2) = match (edge1, edge2) {
            (OneWay(idx), OneWay(other_idx)) => self.swap_one_ways(idx, other_idx),
            (TwoWay(idx1, idx2), TwoWay(other_idx1, other_idx2)) => self.swap_two_ways(idx1, idx2, other_idx1, other_idx2),
            _ => Err(EdgeSwapError::Mismatch(edge1.into(), edge2.into())),
        }?;

        if !self.swappable_edges.remove(&edge1) {
            return Err(EdgeSwapError::NonSwappableEdge(edge1.into()));
        }

        if !self.swappable_edges.remove(&edge2) {
            return Err(EdgeSwapError::NonSwappableEdge(edge2.into()));
        }

        self.swappable_edges.insert(new_edge1);
        self.swappable_edges.insert(new_edge2);

        Ok((new_edge1, new_edge2))
    }

    fn pick_random_edges(&self, rng: &mut impl Rng) -> Option<(SwapEdge, SwapEdge)> {
        use SwapEdge::*;
        let edges = self.swappable_edges.iter();

        let mut buf: Vec<SwapEdge> = Vec::with_capacity(2);
        let num_edges = if rng.get_bool(0.5f64) {
            rng.choose_multiple_fill(
                edges.filter(|edge| matches!(edge, OneWay(x))).cloned(), &mut buf)
        } else {
            rng.choose_multiple_fill(
                edges.filter(|edge| matches!(edge, TwoWay(x, y))).cloned(), &mut buf)
        };

        // If graph does not have enough edges to choose 2 to swap, then return None
        if num_edges != 2 {
            None
        } else {
            Some((buf[0], buf[1]))
        }
    }

    fn get_unreachable_regions(&self) -> Vec<Vec<NodeID>> {
        let condensed_graph = algo::condensation(
            self.base_graph.map(|_, n| n, |_, e| e).into(),
            /* make_acyclic */ true,
        );

        condensed_graph
            .externals(Direction::Incoming)
            .map(|region_idx| condensed_graph.node_weight(region_idx)
                 .expect(&format!(
                         "Region with index {} did not have a node weight", region_idx.index()))
                 .into_iter()
                 .map(|idx| **idx)
                 .collect::<Vec<u32>>())
            .collect()
    }
}
