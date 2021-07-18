use std::{
    collections::{HashMap, HashSet},
    fmt
};
use thiserror::Error;

pub trait ToResult<T, E> {
    fn to_result(self) -> Result<T, E>;
}

#[derive(Hash)]
pub struct EntranceData;
#[derive(Hash)]
pub enum EntranceShuffleType {
    // two-way doors are truly two-way; one-way doors lead to one-way exits
    Standard,
    // two-way doors behave like one-way doors; every door can lead to any exit
    Chaos,
}

#[derive(Hash)]
pub enum Setting {
    EntranceShuffle{ty: EntranceShuffleType, data: EntranceData}
}

#[derive(Error, Debug)]
pub struct NonUniqueSettingsError;

impl fmt::Display for NonUniqueSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: print actual non-unique settings
        write!(f, "Non-unique settings detected: ")
    }
}

pub struct Settings {
    settings: HashSet<Setting>
}

impl Settings {
    pub fn new(settings: &[Setting]) -> Result<Self, NonUniqueSettingsError> {
        let mut unique_settings: HashSet<Setting> = HashSet::new();
        if settings.into_iter().all(move |s| unique_settings.insert(s)) {
            Ok(Self {
                settings: unique_settings
            })
        } else {
            Err(NonUniqueSettingsError)
        }
    }
}

pub type StringID = String;
pub type Address = usize;
pub type Destination = [u8; 4];

pub struct WriteData {
    pub bytes: Vec<u8>,
    pub target_addresses: Vec<usize>
}

pub struct DoorDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}
