use crate::{
    graph::{
        BaseEdgeSwapError, DoorData, EdgeSwapError, GetEdgeEndpointsError, Graph, SwapEdgeIndices,
    },
    rng::{ChooseMultipleFill, RandomBool},
};
use linked_hash_set::LinkedHashSet;
use petgraph::{
    algo,
    graph::{EdgeIndex, NodeIndex},
    stable_graph::StableDiGraph,
    Direction, IntoWeightedEdge,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Address = usize;
type Destination = [u8; 4];
type NodeID = String;

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
    pub door_data: HashMap<NodeID, (Destination, Vec<Address>)>,
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
    door_data: HashMap<NodeID, (Destination, Vec<Address>)>,
    base_graph: StableDiGraph<NodeID, ()>,
    node_map: HashMap<NodeID, NodeIndex>,

    // This needs to be a linked hash set because HashSet iteration order is non-deterministic,
    // which breaks seeded randomization
    swappable_edges: LinkedHashSet<SwapEdge>,
}

fn build_base_graph(
    static_edges: Vec<StaticEdge<NodeID>>,
) -> (StableDiGraph<NodeID, ()>, HashMap<NodeID, NodeIndex>) {
    let mut base_graph_edges: Vec<(NodeID, NodeID)> = vec![];
    for edge in static_edges {
        base_graph_edges.push((edge.start.clone(), edge.end.clone()));
        if edge.two_way {
            base_graph_edges.push((edge.end, edge.start));
        }
    }

    let mut graph = StableDiGraph::new();
    let mut node_map = HashMap::new();

    for edge in base_graph_edges {
        let (start_node_id, end_node_id, _): (NodeID, NodeID, (NodeID, NodeID)) =
            edge.into_weighted_edge();
        insert_edge(&mut graph, &mut node_map, start_node_id, end_node_id);
    }

    (graph, node_map)
}

// Insert an edge between two existing nodes. If nodes do not exist, create nodes and insert edge.
fn insert_edge(
    graph: &mut StableDiGraph<NodeID, ()>,
    node_map: &mut HashMap<NodeID, NodeIndex>,
    a: NodeID,
    b: NodeID,
) -> EdgeIndex {
    let node_idx_a = node_map.get(&a).copied();
    let node_idx_b = node_map.get(&b).copied();
    let (a_idx, b_idx) = match (node_idx_a, node_idx_b) {
        (Some(a_idx), Some(b_idx)) => (a_idx, b_idx),
        (Some(a_idx), None) => (a_idx, add_node(graph, node_map, b)),
        (None, Some(b_idx)) => (add_node(graph, node_map, a), b_idx),
        (None, None) => (add_node(graph, node_map, a), add_node(graph, node_map, b)),
    };
    graph.add_edge(a_idx, b_idx, ())
}

fn add_node(
    graph: &mut StableDiGraph<NodeID, ()>,
    node_map: &mut HashMap<NodeID, NodeIndex>,
    node_id: NodeID,
) -> NodeIndex {
    let node_index = graph.add_node(node_id.clone());
    node_map.insert(node_id, node_index);
    node_index
}

fn add_swappable_edges(
    base_graph: &mut StableDiGraph<NodeID, ()>,
    node_map: &mut HashMap<NodeID, NodeIndex>,
    dynamic_edges: Vec<DynamicEdge<NodeID>>,
) -> LinkedHashSet<SwapEdge> {
    let (two_ways, one_ways): (Vec<DynamicEdge<NodeID>>, Vec<DynamicEdge<NodeID>>) =
        dynamic_edges.into_iter().partition(|e| e.two_way);

    let mut swappable_edges = LinkedHashSet::new();
    for e in one_ways.into_iter() {
        let idx = insert_edge(base_graph, node_map, e.start, e.end);
        swappable_edges.insert(SwapEdge::OneWay(idx));
    }

    for e in two_ways.into_iter() {
        let idx1 = insert_edge(base_graph, node_map, e.start.clone(), e.end.clone());
        let idx2 = insert_edge(base_graph, node_map, e.end, e.start);
        swappable_edges.insert(SwapEdge::TwoWay(idx1, idx2));
    }

    swappable_edges
}

impl GameGraph {
    pub fn new(graph_data: GraphData<NodeID>) -> Self {
        let (mut base_graph, mut node_map) = build_base_graph(graph_data.static_edges);
        let swappable_edges =
            add_swappable_edges(&mut base_graph, &mut node_map, graph_data.dynamic_edges);

        Self {
            door_data: graph_data.door_data,
            base_graph,
            node_map,
            swappable_edges,
        }
    }

    fn edge_node_ids(
        &self,
        idx: EdgeIndex,
    ) -> std::result::Result<(NodeID, NodeID), GetEdgeEndpointsError> {
        let (node1, node2) = self
            .base_graph
            .edge_endpoints(idx)
            .ok_or_else(|| GetEdgeEndpointsError::NoEndpoints(idx.index()))?;
        let node1_id = self
            .base_graph
            .node_weight(node1)
            .ok_or_else(|| GetEdgeEndpointsError::MissingNodeID(idx.index(), node1.index()))?;
        let node2_id = self
            .base_graph
            .node_weight(node2)
            .ok_or_else(|| GetEdgeEndpointsError::MissingNodeID(idx.index(), node2.index()))?;
        Ok((node1_id.clone(), node2_id.clone()))
    }

    fn swap_one_ways(
        &mut self,
        idx: EdgeIndex,
        other_idx: EdgeIndex,
    ) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        let (e1, e2) = self
            .swap_base_graph_edges(idx, other_idx)
            .map_err(EdgeSwapError::BaseEdgeSwap)?;
        Ok((SwapEdge::OneWay(e1), SwapEdge::OneWay(e2)))
    }

    fn swap_two_ways(
        &mut self,
        idx1: EdgeIndex,
        idx2: EdgeIndex,
        other_idx1: EdgeIndex,
        other_idx2: EdgeIndex,
    ) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        let (e1, e2) = self
            .swap_base_graph_edges(idx1, other_idx1)
            .map_err(EdgeSwapError::BaseEdgeSwap)?;
        let (e3, e4) = self
            .swap_base_graph_edges(idx2, other_idx2)
            .map_err(EdgeSwapError::BaseEdgeSwap)?;
        Ok((SwapEdge::TwoWay(e1, e4), SwapEdge::TwoWay(e2, e3)))
    }

    fn swap_base_graph_edges(
        &mut self,
        idx1: EdgeIndex,
        idx2: EdgeIndex,
    ) -> std::result::Result<(EdgeIndex, EdgeIndex), BaseEdgeSwapError> {
        let (edge1a, edge1b) = self
            .edge_node_ids(idx1)
            .map_err(BaseEdgeSwapError::EdgeEndpoints)?;
        let (edge2a, edge2b) = self
            .edge_node_ids(idx2)
            .map_err(BaseEdgeSwapError::EdgeEndpoints)?;

        self.base_graph.remove_edge(idx1).ok_or_else(|| {
            BaseEdgeSwapError::MissingBaseEdge(edge1a.clone(), edge1b.clone(), idx1.index())
        })?;
        self.base_graph.remove_edge(idx2).ok_or_else(|| {
            BaseEdgeSwapError::MissingBaseEdge(edge2a.clone(), edge2b.clone(), idx2.index())
        })?;

        let new_edge_idx1 = insert_edge(&mut self.base_graph, &mut self.node_map, edge1a, edge2b);
        let new_edge_idx2 = insert_edge(&mut self.base_graph, &mut self.node_map, edge2a, edge1b);

        Ok((new_edge_idx1, new_edge_idx2))
    }
}

impl Graph<NodeID, SwapEdge> for GameGraph {
    fn get_edges(&self) -> Vec<(NodeID, NodeID)> {
        let mut res: Vec<(NodeID, NodeID)> = vec![];
        for edge in &self.swappable_edges {
            match edge {
                SwapEdge::OneWay(idx) => res.push(self.edge_node_ids(*idx).unwrap_or_else(|e| {
                    panic!("Error extracting string IDs for one way edge: {}", e)
                })),
                SwapEdge::TwoWay(idx1, idx2) => {
                    res.push(self.edge_node_ids(*idx1).unwrap_or_else(|e| {
                        panic!(
                            "Error extracting string IDs for two way edge (first): {}",
                            e
                        )
                    }));
                    res.push(self.edge_node_ids(*idx2).unwrap_or_else(|e| {
                        panic!(
                            "Error extracting string IDs for two way edge (second): {}",
                            e
                        )
                    }));
                }
            }
        }

        for (start, end) in &res {
            // TODO: Debug log level
            println!("id pair: {}, {}", start, end);
        }

        res
    }

    fn swap_edges(
        &mut self,
        edge1: SwapEdge,
        edge2: SwapEdge,
    ) -> std::result::Result<(SwapEdge, SwapEdge), EdgeSwapError> {
        use SwapEdge::*;
        let (new_edge1, new_edge2) = match (edge1, edge2) {
            (OneWay(idx), OneWay(other_idx)) => self.swap_one_ways(idx, other_idx),
            (TwoWay(idx1, idx2), TwoWay(other_idx1, other_idx2)) => {
                self.swap_two_ways(idx1, idx2, other_idx1, other_idx2)
            }
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

    fn pick_random_edges<R>(&self, rng: &mut R) -> Option<(SwapEdge, SwapEdge)>
    where
        R: RandomBool + ChooseMultipleFill,
    {
        use SwapEdge::*;
        let edges = self.swappable_edges.iter();

        let mut buf: Vec<SwapEdge> = Vec::with_capacity(2);
        let num_edges = if rng.get_bool(0.5f64) {
            rng.choose_multiple_fill(
                edges.filter(|edge| matches!(edge, OneWay(_x))).cloned(),
                &mut buf,
            )
        } else {
            rng.choose_multiple_fill(
                edges.filter(|edge| matches!(edge, TwoWay(_x, _y))).cloned(),
                &mut buf,
            )
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
            .map(|region_idx| {
                condensed_graph
                    .node_weight(region_idx)
                    .unwrap_or_else(|| {
                        panic!(
                            "Region with index {} did not have a node weight",
                            region_idx.index()
                        )
                    })
                    .iter()
                    .map(|idx| (*idx).clone())
                    .collect::<Vec<NodeID>>()
            })
            .collect()
    }
}

impl DoorData<NodeID> for GameGraph {
    fn door_data(&self) -> &HashMap<NodeID, (Destination, Vec<Address>)> {
        &self.door_data
    }
}
