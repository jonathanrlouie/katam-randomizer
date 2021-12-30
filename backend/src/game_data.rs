use std::{
    fs::File,
    collections::HashMap,
};
use serde::{Deserialize, Serialize};

type Address = usize;
type StringID = String;
type Destination = [u8; 4];

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
struct StaticEdge<IDType> {
    start: IDType,
    end: IDType,
    two_way: bool,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
struct DynamicEdge<IDType> {
    start: IDType,
    end: IDType,
    two_way: bool,
}

#[derive(Serialize, Deserialize)]
struct GraphData<IDType> {
    door_data: HashMap<StringID, (Destination, Vec<Address>)>,
    static_edges: Vec<StaticEdge<IDType>>,
    dynamic_edges: Vec<DynamicEdge<IDType>>,
}

// maps for converting randomized game data back into ROM addresses
pub struct RomDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}

pub struct GameData {
    pub graph: GameGraph,
    pub rom_data_maps: RomDataMaps,
}

fn build_rom_data_maps(graph_data: &mut GraphData<StringID>) -> anyhow::Result<RomDataMaps> {
    // for each dyn edge: map endpoint of dynamic_edge to start's destination and start
    // to start's addresses to replace
    let mut start_map: HashMap<StringID, Vec<Address>> = HashMap::new();
    let mut end_map: HashMap<StringID, Destination> = HashMap::new();
    
    for edge in graph_data.dynamic_edges {
        println!("Dyn Edge: {}, {}", edge.start, edge.end);
        if edge.two_way {
            let (end_destination, start_addresses) = graph_data.door_data.remove(&edge.start).ok_or_else(|| Error::MissingDoorDataNode(edge.start.clone()))?;
            let (start_destination, end_addresses) = graph_data.door_data.remove(&edge.end).ok_or_else(|| Error::MissingDoorDataNode(edge.end.clone()))?;
    
            start_map.insert(edge.start.clone(), start_addresses);
            end_map.insert(edge.end.clone(), end_destination);
    
            start_map.insert(edge.end.clone(), end_addresses);
            end_map.insert(edge.start.clone(), start_destination);
        } else {
            let (end_destination, start_addresses) = graph_data.door_data.remove(&edge.start).ok_or_else(|| Error::MissingDoorDataNode(edge.start.clone()))?;
    
            start_map.insert(edge.start.clone(), start_addresses);
            end_map.insert(edge.end.clone(), end_destination);
        }
    }
    
    Ok(RomDataMaps {
        start_map,
        end_map
    })
}

pub fn load_game_data(path: &str) -> Result<GameData> {
    let mut file = File::open(path)?;
    let graph_data: GraphData<StringID> = serde_json::from_reader(file).map_err(Error::from)?;
    let rom_data_maps = build_rom_data_maps(&graph_data)?;
    let graph = GameGraph::new(graph_data);
    Ok(GameData {
        graph,
        rom_data_maps,
    })
}
