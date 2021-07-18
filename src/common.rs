use std::collections::HashMap;
use anyhow;

#[derive(Hash)]
pub struct EntranceData;
#[derive(Hash)]
pub enum EntranceShuffleType {
    // two-way doors are truly two-way; one-way doors lead to one-way exits
    pub Standard,
    // two-way doors behave like one-way doors; every door can lead to any exit
    pub Chaos,
}

#[derive(Hash)]
pub enum Setting {
    pub EntranceShuffle{ty: EntranceShuffleType, data: EntranceData}
}

pub struct Settings {
    settings: HashSet<Setting>
}

impl Settings {
    pub fn new(settings: &[Setting]) -> anyhow::Result<Self> {
        let mut unique_settings: HashSet<Setting> = HashSet::new();
        if settings.into_iter().all(move |s| unique_settings.insert(s)) {
            Ok(Self {
                settings: unique_settings
            })
        } else {
            Err()
        }
    }
}

pub type StringID = String;
type Address = usize;
type Destination = [u8; 4];

pub struct WriteData {
    pub bytes: Vec<u8>,
    pub target_addresses: Vec<usize>
}

pub struct DoorDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}
