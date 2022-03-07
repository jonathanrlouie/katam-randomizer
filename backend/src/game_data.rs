use crate::{error::RomDataMapError, game_graph, rom::RomDataMaps};
use std::{collections::HashMap, fs::File};

type Address = usize;
type Destination = [u8; 4];
type StringID = String;

#[derive(Clone)]
pub struct GameData {
    pub graph: game_graph::GameGraph,
    pub rom_data_maps: RomDataMaps,
}

impl GameData {
    pub fn load_game_data(path: &str) -> Self {
        let file = File::open(path).expect("Error opening KatAM game data file.");
        let mut graph_data: game_graph::GraphData<StringID> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("Error deserializing KatAM game data: {}", e));
        let rom_data_maps = build_rom_data_maps(&mut graph_data)
            .unwrap_or_else(|e| panic!("Error building ROM data maps: {}", e));
        let graph = game_graph::GameGraph::new(graph_data);
        Self {
            graph,
            rom_data_maps,
        }
    }
}

fn build_rom_data_maps(
    graph_data: &mut game_graph::GraphData<StringID>,
) -> std::result::Result<RomDataMaps, RomDataMapError> {
    // for each dyn edge: map endpoint of dynamic_edge to start's destination and start
    // to start's addresses to replace
    let mut start_map: HashMap<StringID, Vec<Address>> = HashMap::new();
    let mut end_map: HashMap<StringID, Destination> = HashMap::new();

    for edge in &graph_data.dynamic_edges {
        // TODO: Debug level logging
        println!("Dyn Edge: {}, {}", edge.start, edge.end);
        if edge.two_way {
            let (end_destination, start_addresses) = graph_data
                .door_data
                .remove(&edge.start)
                .ok_or_else(|| RomDataMapError::MissingDoorData(edge.start.clone()))?;
            let (start_destination, end_addresses) = graph_data
                .door_data
                .remove(&edge.end)
                .ok_or_else(|| RomDataMapError::MissingDoorData(edge.end.clone()))?;

            start_map.insert(edge.start.clone(), start_addresses);
            end_map.insert(edge.end.clone(), end_destination);

            start_map.insert(edge.end.clone(), end_addresses);
            end_map.insert(edge.start.clone(), start_destination);
        } else {
            let (end_destination, start_addresses) = graph_data
                .door_data
                .remove(&edge.start)
                .ok_or_else(|| RomDataMapError::MissingDoorData(edge.start.clone()))?;

            start_map.insert(edge.start.clone(), start_addresses);
            end_map.insert(edge.end.clone(), end_destination);
        }
    }

    Ok(RomDataMaps { start_map, end_map })
}
