use std::{collections::HashMap, fmt};
use thiserror::Error;

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

#[derive(Debug)]
pub struct EntranceData;

#[derive(Debug)]
pub enum EntranceShuffleType {
    // two-way doors are truly two-way; one-way doors lead to one-way exits
    Standard,
    // two-way doors behave like one-way doors; every door can lead to any exit
    Chaos,
}

#[derive(Debug)]
pub enum Setting {
    EntranceShuffle {
        ty: EntranceShuffleType,
        data: EntranceData,
    },
}

impl Setting {
    fn get_key(&self) -> String {
        use Setting::*;
        match self {
            EntranceShuffle { .. } => "EntranceShuffle",
        }
        .to_string()
    }
}

#[derive(Error, Debug)]
pub struct NonUniqueSettingsError {
    duplicates: Vec<String>,
}

impl fmt::Display for NonUniqueSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Non-unique settings detected: {:?}", self.duplicates)
    }
}

pub struct Settings {
    settings: Vec<Setting>,
}

impl IntoIterator for Settings {
    type Item = Setting;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.settings.into_iter()
    }
}

impl Settings {
    pub fn new(settings: Vec<Setting>) -> Result<Self, NonUniqueSettingsError> {
        let mut setting_keys: Vec<String> = settings
            .iter()
            .map(|s| s.get_key())
            .to_owned()
            .collect::<Vec<String>>();
        setting_keys.sort();
        setting_keys.dedup();

        if !setting_keys.is_empty() {
            Ok(Self { settings })
        } else {
            Err(NonUniqueSettingsError {
                duplicates: setting_keys,
            })
        }
    }
}

pub type StringID = String;
pub type Address = usize;
pub type Destination = [u8; 4];

pub struct WriteData {
    pub bytes: Vec<u8>,
    pub target_addresses: Vec<usize>,
}

pub struct DoorDataMaps {
    pub start_map: HashMap<StringID, Vec<Address>>,
    pub end_map: HashMap<StringID, Destination>,
}
